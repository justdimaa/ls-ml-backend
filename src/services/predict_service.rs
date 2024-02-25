use std::{sync::Arc, time::Instant};

use actix_web::{post, web, Responder};
use image::{imageops::FilterType, GenericImageView};
use ndarray::{Array, IxDyn};
use tokio::task::JoinSet;

use crate::{api, config, state::AppState, MODEL_VERSION};

const IOU_THRESHOLD: f32 = 0.7;
const YOLOV8_IMG_SIZE: u32 = 640;

#[post("/predict")]
pub async fn predict(
    req: web::Json<api::PredictRequest>,
    data: web::Data<AppState>,
    cfg: web::Data<config::Config>,
) -> actix_web::Result<impl Responder> {
    let req = req.into_inner();
    let project_id = api::get_project_id(&req.project)
        .ok_or(actix_web::error::ErrorBadRequest("invalid project id"))?;

    let model = data
        .models
        .read()
        .await
        .get(&project_id)
        .ok_or(actix_web::error::ErrorNotFound(format!(
            "model for project id {} not found",
            project_id,
        )))?
        .clone();

    let tasks_n = req.tasks.len();

    let mut join_set = JoinSet::new();

    let img_client = Arc::new(reqwest::Client::new());

    for (idx, task) in req.tasks.into_iter().enumerate() {
        let ls_url = cfg.label_studio_url.clone();
        let ls_token = cfg.label_studio_token.clone();
        let labels = cfg.ml_labels.clone();

        let model = model.clone();
        let img_client = img_client.clone();

        join_set.spawn(async move {
            let img_resp = img_client
                .get(format!("{}{}", ls_url, task.data.image))
                .header("Authorization", format!("Token {}", ls_token))
                .send()
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))
                .unwrap();

            let img_buf = img_resp
                .bytes()
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))
                .unwrap();

            let (input, img_width, img_height) = prepare_input(&img_buf).unwrap();
            let output = run_model(&model, input).unwrap();

            let boxes = process_output(output, &labels);
            let boxes = deduplicate_boxes(boxes);

            let score = boxes.iter().map(|(_, _, s)| s).sum::<f32>() / boxes.len().max(1) as f32;

            (
                idx,
                api::PredictResponse {
                    result: boxes
                        .into_iter()
                        .enumerate()
                        .map(|(i, (b, label, p))| api::Pred {
                            id: i.to_string(),
                            from_name: "label".to_string(),
                            to_name: "image".to_string(),
                            original_width: img_width,
                            original_height: img_height,
                            image_rotation: 0,
                            value: api::PredValue {
                                rotation: 0,
                                x: b.xc - b.w / 2.0,
                                y: b.yc - b.h / 2.0,
                                width: b.w,
                                height: b.h,
                                rectanglelabels: vec![label.to_string()],
                            },
                            score: p,
                            t: "rectanglelabels".to_string(),
                            readonly: false,
                        })
                        .collect(),
                    score,
                    model_version: Some(MODEL_VERSION.to_string()),
                },
            )
        });
    }

    let mut results = vec![];
    let total_sw = Instant::now();

    while let Some(Ok(join)) = join_set.join_next().await {
        results.push(join);
    }

    tracing::info!(
        "predicted {} tasks for project id {} in {}ms",
        tasks_n,
        project_id,
        total_sw.elapsed().as_millis()
    );

    results.sort_by(|(a, _), (b, _)| a.cmp(&b));

    let results = results
        .into_iter()
        .map(|(_, result)| result)
        .collect::<Vec<_>>();

    Ok(web::Json(serde_json::json!({ "results": results })))
}

fn prepare_input(buf: &[u8]) -> Result<(Array<f32, IxDyn>, u32, u32), actix_web::Error> {
    let img = image::load_from_memory(&buf).map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    let (img_width, img_height) = (img.width(), img.height());
    let img = img.resize_exact(YOLOV8_IMG_SIZE, YOLOV8_IMG_SIZE, FilterType::CatmullRom);

    let mut input =
        Array::zeros((1, 3, YOLOV8_IMG_SIZE as usize, YOLOV8_IMG_SIZE as usize)).into_dyn();

    for pixel in img.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let [r, g, b, _] = pixel.2 .0;
        input[[0, 0, y, x]] = (r as f32) / 255.0;
        input[[0, 1, y, x]] = (g as f32) / 255.0;
        input[[0, 2, y, x]] = (b as f32) / 255.0;
    }

    Ok((input, img_width, img_height))
}

fn run_model(
    model: &ort::Session,
    input: Array<f32, IxDyn>,
) -> Result<Array<f32, IxDyn>, actix_web::Error> {
    let outputs: ort::SessionOutputs =
        model
            .run(ort::inputs!["images" => input.view()].unwrap())
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let output = outputs["output0"]
        .extract_tensor::<f32>()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .view()
        .t()
        .into_owned();

    Ok(output)
}

fn process_output(
    output: Array<f32, IxDyn>,
    labels: &Vec<String>,
) -> Vec<(BoundingBox, &String, f32)> {
    let mut result = Vec::new();

    let output = output.slice(ndarray::s![.., .., 0]);

    for row in output.axis_iter(ndarray::Axis(0)) {
        let row: Vec<_> = row.iter().copied().collect();
        let (class_id, prob) = row
            .iter()
            // skip bounding box coordinates
            .skip(4)
            .enumerate()
            .map(|(index, value)| (index, *value))
            .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
            .unwrap();

        if prob < 0.5 {
            continue;
        }

        let Some(label) = labels.get(class_id) else {
            continue;
        };

        result.push((
            BoundingBox {
                xc: row[0] / YOLOV8_IMG_SIZE as f32 * 100.0,
                yc: row[1] / YOLOV8_IMG_SIZE as f32 * 100.0,
                w: row[2] / YOLOV8_IMG_SIZE as f32 * 100.0,
                h: row[3] / YOLOV8_IMG_SIZE as f32 * 100.0,
            },
            label,
            prob,
        ));
    }

    result
}

// removes duplicate boxes for the same object using non maximum suppression
fn deduplicate_boxes(
    mut boxes: Vec<(BoundingBox, &String, f32)>,
) -> Vec<(BoundingBox, &String, f32)> {
    let mut result = vec![];

    boxes.sort_by(|(_, _, p1), (_, _, p2)| p2.total_cmp(&p1));

    while boxes.len() > 0 {
        result.push(boxes[0]);
        boxes = boxes
            .iter()
            .filter(|(box1, _, _)| boxes[0].0.iou(&box1) < IOU_THRESHOLD)
            .map(|x| *x)
            .collect()
    }

    result
}

#[derive(Debug, Clone, Copy)]
struct BoundingBox {
    pub xc: f32,
    pub yc: f32,
    pub w: f32,
    pub h: f32,
}

impl BoundingBox {
    pub fn intersection(&self, other: &BoundingBox) -> f32 {
        let x1 = (self.xc - self.w / 2.0).max(other.xc - other.w / 2.0);
        let y1 = (self.yc - self.h / 2.0).max(other.yc - other.h / 2.0);
        let x2 = (self.xc + self.w / 2.0).max(other.xc + other.w / 2.0);
        let y2 = (self.yc + self.h / 2.0).max(other.yc + other.h / 2.0);
        (x2 - x1) * (y2 - y1)
    }

    pub fn union(&self, other: &BoundingBox) -> f32 {
        let box1_area = self.w * self.h;
        let box2_area = other.w * other.h;
        box1_area + box2_area - self.intersection(other)
    }

    pub fn iou(&self, other: &BoundingBox) -> f32 {
        self.intersection(other) / self.union(other)
    }
}

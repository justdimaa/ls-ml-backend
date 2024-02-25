
# ML Backend for Label Studio

This is an ML backend for Label Studio written in Rust. It allows you to integrate your models with Label Studio to assist workflows such as pre-annotation and interactive labeling.

## Features

- Supported providers:
  - CPU
  - CUDA (for GPU acceleration)
- Supported models:
  - [YOLOv8](https://github.com/ultralytics/ultralytics)
  - Please let me know if other models work too by opening a new issue
- Supported predictions:
  - Object detection
  - (Work in progress) Image segmentation


## Future development

- Model training: Train your models directly from Label Studio annotations.
- Web UI: Manage models (upload, download, configure) through a web interface.


## Run Locally (Docker, CPU)

Clone the project

```bash
  $ git clone https://github.com/justdimaa/ls-ml-backend
```

Go to the project directory

```bash
  $ cd ls-ml-backend
```


Create a new folder called `models`

```bash
  $ mkdir models
```

Move your model to the folder

- Verify your model file is saved in the ONNX format.
-  Rename the model file to match the Project ID of your Label Studio project. You can find the Project ID in the URL of your Label Studio project: `https://yourlabelstudio.site/projects/PROJECT_ID/data`.

```bash
  $ mv path/to/your/model.onnx models/PROJECT_ID.onnx
```
Example with Project ID 4:

```
../
|- src/
|- models/
   |- 4.onnx
...
```

Change the environment variables (rename file `.env.example` to `.env`)
```bash
  $ mv .env.example .env
```

Start the service

```bash
  $ docker compose up -d
```


## Run Locally (Bare metal, CUDA)

A Dockerfile with CUDA support is still under development.
You can still run the backend locally without Docker.


⚠️ **Prerequisites: Ensure you have the following CUDA packages installed: `cudatoolkit` `cublas` `cudnn` `cudart`, otherwise the backend will default to the CPU provider.** ⚠️

Repeat everything from `Run Locally (Docker, CPU)`, up to the point where you set your environment variables. To enable CUDA in the backend, change the variable to `ML_PROVIDER=cuda`.


Create a new folder called `lib` and extract the onnxruntime into the folder 

```bash
  $ mkdir lib \
    && wget https://github.com/microsoft/onnxruntime/releases/download/v1.17.0/onnxruntime-linux-x64-cuda12-1.17.0.tgz \
    && tar -xvzf onnxruntime-linux-x64-cuda12-1.17.0.tgz \
    && rm -f onnxruntime-linux-x64-cuda12-1.17.0.tgz \
    && mv onnxruntime-linux-x64-cuda-1.17.0/ lib/
```

Export the environment variable for the dynamically linked library

```bash
  $ export ORT_DYLIB_PATH=${PWD}/lib/onnxruntime-linux-x64-cuda-1.17.0/lib/libonnxruntime.so
```

Then start the service with
```bash
  $ cargo run --release
```


## Environment Variables

To run this project, you will need to make changes to the following environment variables in your `.env` file:

`LABEL_STUDIO_URL` Change this to the Label Studio url under that the backend can reach the instance.

`LABEL_STUDIO_TOKEN` Change this to your account's Label Studio token that the backend can use to access the instance data.

`ML_LABELS` Change this to the labels used in your project, separating them with commas (e.g., `player,chest,chicken`)


## Connect to Label Studio

Go to the Label Studio project `Settings > Machine Learning`, and click on `Add Model`. Enter the backend url, click on `Validate and Save`.

For now, it will only connect successfully, if you are using the `Object Detection with Bounding Boxes` template under `Settings > Labeling Interface > Computer Vision`.

If Label Studio can't find your backend, please ensure that you can establish a connection to the backend from the Label Studio instance.
```bash
  $ curl http://yourmlbackend:9090/health
  {"status":"UP"}
```

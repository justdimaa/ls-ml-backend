{ pkgs ? import <nixpkgs> {} }:

let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  ruststable = (pkgs.latest.rustChannels.stable.default.override {
    extensions = [
      "rust-src"
    ];
  });
in pkgs.mkShell {
  buildInputs = with pkgs; [
    ruststable
  ];
  
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [
    pkgs.stdenv.cc.cc
  ]}";
  ORT_DYLIB_PATH = "lib/onnxruntime-linux-x64-1.17.0/lib/libonnxruntime.so";
}

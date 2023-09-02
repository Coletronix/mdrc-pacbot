extern crate protoc_rust;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/robomodules/protos")
        .inputs(&[
            "src/robomodules/definitions/lightState.proto",
            "src/robomodules/definitions/pacmanState.proto",
        ])
        // .include("path/to/your")
        .run()
        .expect("protoc");
}

extern crate protoc_rust;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/protobuf/protos")
        .inputs(&[
            "src/protobuf/definitions/lightState.proto",
            "src/protobuf/definitions/pacmanState.proto",
        ])
        // .include("path/to/your")
        .run()
        .expect("protoc");
}

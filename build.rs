extern crate protoc_rust;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/robomodules/protos")
        .inputs(&[
            "src/robomodules/definitions/light_state.proto",
            "src/robomodules/definitions/pacman_state.proto",
            "src/robomodules/definitions/subscribe.proto",
        ])
        // .include("path/to/your")
        .run()
        .expect("protoc");
}

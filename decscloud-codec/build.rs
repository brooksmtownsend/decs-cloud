fn main() {
    prost_build::compile_protos(&["src/timer.proto"], &["src/"]).unwrap();
}

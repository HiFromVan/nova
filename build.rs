fn main() {
    tonic_build::compile_protos("proto/simulator.proto").unwrap();
}

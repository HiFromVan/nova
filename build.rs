fn main() {
    tonic_build::compile_protos("proto/simulator.proto").unwrap();
    tonic_build::compile_protos("proto/nova_control.proto").unwrap();
}

mod brain_interface;

pub use brain_interface::{BrainInterface, BaselineGait, SensorData, MotorCommands};

fn main() {
    println!("Nova — use `cargo run --example grpc_client` to test gRPC communication.");
}

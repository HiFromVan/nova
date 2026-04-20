mod baseline;
mod types;

pub use baseline::BaselineGait;
pub use types::{MotorCommands, SensorData};

/// Every brain implementation must implement this trait.
pub trait BrainInterface: Send + Sync {
    fn decide(&mut self, sensors: &SensorData, dt: f32) -> MotorCommands;
    fn reset(&mut self);
    fn name(&self) -> &str {
        "BrainInterface"
    }
}

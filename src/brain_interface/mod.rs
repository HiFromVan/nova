mod baseline;
mod types;

pub use baseline::BaselineGait;
pub use types::{MotorCommands, SensorData};

pub trait BrainInterface: Send + Sync {
    fn decide(&mut self, sensors: &SensorData, dt: f32) -> MotorCommands;
    fn reset(&mut self);
    fn name(&self) -> &str { "BrainInterface" }
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> { None }
}

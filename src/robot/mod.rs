use async_trait::async_trait;
use crate::brain_interface::{SensorData, MotorCommands};

#[async_trait]
pub trait RobotIO: Send {
    async fn reset(&mut self) -> SensorData;
    /// 发送指令，返回执行后的传感器数据
    async fn step(&mut self, cmd: &MotorCommands) -> SensorData;
}

pub mod sim;
pub mod unitree;

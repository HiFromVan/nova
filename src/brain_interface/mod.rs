//! Brain Interface - 连接 Becoming 决策系统与 Nova 执行系统
//!
//! 这个模块定义了决策系统（Becoming）与执行系统（Nova）之间的接口。
//!
//! 架构：
//! ```
//! Becoming (决策大脑)
//!     ↓ 通过 BrainInterface
//! Nova (执行层)
//!     ↓
//! 仿真器/真机
//! ```

use bevy::prelude::*;

mod baseline;
mod types;

pub use baseline::BaselineGait;
pub use types::*;

/// Brain Interface - 所有决策系统必须实现的接口
///
/// 这个 trait 定义了决策系统的标准接口：
/// - 输入：传感器数据（SensorData）
/// - 输出：电机指令（MotorCommands）
///
/// 实现者可以是：
/// - 规则步态（BaselineGait）
/// - 神经网络（NeuralBrain）
/// - Becoming 系统（BecomingBrain）
pub trait BrainInterface: Send + Sync {
    /// 决策函数：根据传感器数据输出电机指令
    ///
    /// # 参数
    /// - `sensors`: 当前的传感器数据
    /// - `dt`: 时间步长（秒）
    ///
    /// # 返回
    /// 电机指令（关节目标角度或力矩）
    fn decide(&mut self, sensors: &SensorData, dt: f32) -> MotorCommands;

    /// 重置决策系统状态
    fn reset(&mut self);

    /// 获取决策系统的名称（用于调试）
    fn name(&self) -> &str {
        "BrainInterface"
    }
}

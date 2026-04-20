//! Baseline Gait - 规则步态作为基准
//!
//! 这是一个简单的规则步态生成器，作为 Becoming 的 baseline。
//! 使用正弦波生成周期性的步态。

use super::{BrainInterface, MotorCommands, SensorData, JointIndex};
use bevy::prelude::*;

/// Baseline 步态生成器
///
/// 使用简单的正弦波生成周期性步态。
/// 这是最基础的实现，用于：
/// 1. 验证接口设计
/// 2. 作为对比基准
/// 3. 调试和测试
pub struct BaselineGait {
    /// 步态相位（0 到 2π）
    phase: f32,

    /// 步频（Hz）
    frequency: f32,

    /// 目标速度（从键盘输入）
    target_velocity: Vec3,

    /// 步幅（米）
    stride_length: f32,

    /// 步态参数
    config: GaitConfig,
}

/// 步态配置参数
#[derive(Debug, Clone)]
pub struct GaitConfig {
    /// 髋关节摆动幅度（弧度）
    pub hip_swing_amp: f32,

    /// 膝关节弯曲幅度（弧度）
    pub knee_bend_amp: f32,

    /// 手臂摆动幅度（弧度）
    pub arm_swing_amp: f32,

    /// 步频（Hz）
    pub step_frequency: f32,
}

impl Default for GaitConfig {
    fn default() -> Self {
        Self {
            hip_swing_amp: 0.4,
            knee_bend_amp: 0.6,
            arm_swing_amp: 0.3,
            step_frequency: 1.0,  // 1 Hz = 1 步/秒
        }
    }
}

impl BaselineGait {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            frequency: 1.0,
            target_velocity: Vec3::ZERO,
            stride_length: 0.5,
            config: GaitConfig::default(),
        }
    }

    pub fn with_config(config: GaitConfig) -> Self {
        Self {
            phase: 0.0,
            frequency: config.step_frequency,
            target_velocity: Vec3::ZERO,
            stride_length: 0.5,
            config,
        }
    }

    /// 设置目标速度（从键盘输入）
    pub fn set_target_velocity(&mut self, velocity: Vec3) {
        self.target_velocity = velocity;
    }

    /// 生成步态
    fn generate_gait(&mut self, dt: f32) -> MotorCommands {
        // 如果没有目标速度，保持站立姿态
        if self.target_velocity.length() < 0.01 {
            return MotorCommands {
                joint_targets: vec![0.0; 8],
                joint_torques: None,
                desired_velocity: Vec3::ZERO,
            };
        }

        // 更新相位
        self.phase += 2.0 * std::f32::consts::PI * self.frequency * dt;
        if self.phase > 2.0 * std::f32::consts::PI {
            self.phase -= 2.0 * std::f32::consts::PI;
        }

        // 生成周期性步态
        let s = self.phase.sin();
        let c = self.phase.cos();

        // 腿部：左右腿相位相反
        let left_hip = s * self.config.hip_swing_amp;
        let right_hip = -s * self.config.hip_swing_amp;

        // 膝盖：只在摆动相弯曲
        let left_knee = if left_hip > 0.0 {
            left_hip.abs() * self.config.knee_bend_amp
        } else {
            0.0
        };
        let right_knee = if right_hip > 0.0 {
            right_hip.abs() * self.config.knee_bend_amp
        } else {
            0.0
        };

        // 手臂：与对侧腿同步摆动
        let left_arm = -s * self.config.arm_swing_amp;
        let right_arm = s * self.config.arm_swing_amp;

        let mut commands = MotorCommands::default();
        commands.joint_targets[JointIndex::LeftHip.as_usize()] = left_hip;
        commands.joint_targets[JointIndex::LeftKnee.as_usize()] = left_knee;
        commands.joint_targets[JointIndex::RightHip.as_usize()] = right_hip;
        commands.joint_targets[JointIndex::RightKnee.as_usize()] = right_knee;
        commands.joint_targets[JointIndex::LeftShoulder.as_usize()] = left_arm;
        commands.joint_targets[JointIndex::RightShoulder.as_usize()] = right_arm;
        commands.desired_velocity = self.target_velocity;

        commands
    }
}

impl BrainInterface for BaselineGait {
    fn decide(&mut self, _sensors: &SensorData, dt: f32) -> MotorCommands {
        // 简单的规则步态，不需要传感器反馈
        // 未来可以加入平衡控制、地形适应等
        self.generate_gait(dt)
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.target_velocity = Vec3::ZERO;
    }

    fn name(&self) -> &str {
        "BaselineGait"
    }
}

impl Default for BaselineGait {
    fn default() -> Self {
        Self::new()
    }
}

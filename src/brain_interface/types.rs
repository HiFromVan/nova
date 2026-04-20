//! 数据类型定义 - Brain Interface 使用的数据结构

use bevy::prelude::*;

/// 传感器数据 - Nova 采集并发送给 Becoming
///
/// 这个结构包含了机器人的所有状态信息。
/// 第一阶段：简化版（位置、速度、姿态）
/// 未来可扩展：摄像头、激光雷达等
#[derive(Debug, Clone)]
pub struct SensorData {
    /// 时间戳（秒）
    pub timestamp: f64,

    /// 躯干位置（世界坐标系）
    pub position: Vec3,

    /// 躯干速度（世界坐标系）
    pub velocity: Vec3,

    /// 躯干姿态（四元数）
    pub orientation: Quat,

    /// 躯干角速度
    pub angular_velocity: Vec3,

    /// 关节角度（弧度）
    /// 顺序：[左髋, 左膝, 右髋, 右膝, 左肩, 左肘, 右肩, 右肘]
    pub joint_angles: Vec<f32>,

    /// 关节速度（弧度/秒）
    pub joint_velocities: Vec<f32>,

    /// 足部接触状态 [左脚, 右脚]
    pub foot_contacts: [bool; 2],

    /// 是否稳定（用于安全检查）
    pub is_stable: bool,
}

impl Default for SensorData {
    fn default() -> Self {
        Self {
            timestamp: 0.0,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            angular_velocity: Vec3::ZERO,
            joint_angles: vec![0.0; 8],
            joint_velocities: vec![0.0; 8],
            foot_contacts: [false, false],
            is_stable: true,
        }
    }
}

/// 电机指令 - Becoming 输出，Nova 执行
///
/// 这个结构定义了机器人应该执行的动作。
/// 可以是目标角度（位置控制）或目标力矩（力控制）
#[derive(Debug, Clone)]
pub struct MotorCommands {
    /// 关节目标角度（弧度）
    /// 顺序：[左髋, 左膝, 右髋, 右膝, 左肩, 左肘, 右肩, 右肘]
    pub joint_targets: Vec<f32>,

    /// 可选：关节目标力矩（牛米）
    /// 如果为 None，使用位置控制
    pub joint_torques: Option<Vec<f32>>,

    /// 期望的躯干速度（用于高层控制）
    pub desired_velocity: Vec3,
}

impl Default for MotorCommands {
    fn default() -> Self {
        Self {
            joint_targets: vec![0.0; 8],
            joint_torques: None,
            desired_velocity: Vec3::ZERO,
        }
    }
}

/// 关节索引 - 方便访问关节数据
#[derive(Debug, Clone, Copy)]
pub enum JointIndex {
    LeftHip = 0,
    LeftKnee = 1,
    RightHip = 2,
    RightKnee = 3,
    LeftShoulder = 4,
    LeftElbow = 5,
    RightShoulder = 6,
    RightElbow = 7,
}

impl JointIndex {
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

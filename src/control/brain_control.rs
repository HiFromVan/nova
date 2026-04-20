//! Brain Control System - 通过 BrainInterface 控制机器人
//!
//! 这个系统是 Nova 的核心控制循环：
//! 1. 采集传感器数据
//! 2. 调用 BrainInterface 决策
//! 3. 执行电机指令

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::brain_interface::{BrainInterface, SensorData, MotorCommands, BaselineGait};
use crate::models::humanoid::{Torso, JointTargets, Joint, MotorTarget};

/// Brain 控制系统
///
/// 这是主控制循环，连接 Becoming 和 Nova
pub fn brain_control_system(
    time: Res<Time>,
    mut brain_query: Query<&mut BaselineGait>,
    torso_query: Query<(&Transform, &Velocity), With<Torso>>,
    joint_query: Query<(&Joint, &MotorTarget)>,
    mut targets_query: Query<&mut JointTargets, With<Torso>>,
) {
    // 获取 Brain
    let Ok(mut brain) = brain_query.get_single_mut() else {
        return;
    };

    // 1. 采集传感器数据
    let sensors = collect_sensor_data(&torso_query, &joint_query);

    // 2. Brain 决策
    let dt = time.delta_secs();
    let commands = brain.decide(&sensors, dt);

    // 3. 执行指令（只在有目标速度时更新，否则保持站立姿态）
    if commands.desired_velocity.length() > 0.01 {
        println!("执行 Brain 指令: 速度={:.2}", commands.desired_velocity.length());
        execute_commands(commands, &mut targets_query);
    }
}

/// 采集传感器数据
fn collect_sensor_data(
    torso_query: &Query<(&Transform, &Velocity), With<Torso>>,
    joint_query: &Query<(&Joint, &MotorTarget)>,
) -> SensorData {
    let mut sensors = SensorData::default();

    // 采集躯干状态
    if let Ok((transform, velocity)) = torso_query.get_single() {
        sensors.position = transform.translation;
        sensors.orientation = transform.rotation;
        sensors.velocity = velocity.linvel;
        sensors.angular_velocity = velocity.angvel;
    }

    // 采集关节状态
    for (joint, motor_target) in joint_query.iter() {
        let index = match joint {
            Joint::LeftHipPitch => 0,
            Joint::LeftKnee => 1,
            Joint::RightHipPitch => 2,
            Joint::RightKnee => 3,
            Joint::LeftShoulderPitch => 4,
            Joint::LeftElbow => 5,
            Joint::RightShoulderPitch => 6,
            Joint::RightElbow => 7,
            _ => continue,
        };

        if index < sensors.joint_angles.len() {
            sensors.joint_angles[index] = motor_target.0;
        }
    }

    sensors
}

/// 执行电机指令
fn execute_commands(
    commands: MotorCommands,
    targets_query: &mut Query<&mut JointTargets, With<Torso>>,
) {
    let Ok(mut targets) = targets_query.get_single_mut() else {
        return;
    };

    // 更新关节目标
    if commands.joint_targets.len() >= 8 {
        targets.l_hip_pitch = commands.joint_targets[0];
        targets.l_knee = commands.joint_targets[1];
        targets.r_hip_pitch = commands.joint_targets[2];
        targets.r_knee = commands.joint_targets[3];
        targets.l_shoulder_pitch = commands.joint_targets[4];
        targets.l_elbow = commands.joint_targets[5];
        targets.r_shoulder_pitch = commands.joint_targets[6];
        targets.r_elbow = commands.joint_targets[7];

        println!("关节目标: L髋={:.2}, L膝={:.2}, R髋={:.2}, R膝={:.2}",
                 targets.l_hip_pitch, targets.l_knee,
                 targets.r_hip_pitch, targets.r_knee);
    }
}

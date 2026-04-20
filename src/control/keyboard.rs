//! 键盘输入系统 - 将键盘输入转换为目标速度

use bevy::prelude::*;
use crate::brain_interface::BaselineGait;

/// 键盘输入系统
///
/// 读取 WASD 键盘输入，转换为目标速度，
/// 然后传递给 BrainInterface
pub fn keyboard_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gait_query: Query<&mut BaselineGait>,
) {
    let Ok(mut gait) = gait_query.get_single_mut() else {
        return;
    };

    // 读取键盘输入
    let forward = keyboard.pressed(KeyCode::KeyW) as i32 as f32;
    let backward = keyboard.pressed(KeyCode::KeyS) as i32 as f32;
    let left = keyboard.pressed(KeyCode::KeyA) as i32 as f32;
    let right = keyboard.pressed(KeyCode::KeyD) as i32 as f32;

    // 计算目标速度
    let forward_speed = (forward - backward) * 0.5;  // 前后速度
    let lateral_speed = (right - left) * 0.3;        // 左右速度

    let target_velocity = Vec3::new(forward_speed, 0.0, lateral_speed);

    // 调试：打印键盘输入
    if target_velocity.length() > 0.01 {
        println!("键盘输入: 前后={:.2}, 左右={:.2}", forward_speed, lateral_speed);
    }

    // 设置到 BaselineGait
    gait.set_target_velocity(target_velocity);
}

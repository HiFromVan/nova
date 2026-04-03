//! 控制模块 - 负责机器人运动控制

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// 相机控制器组件
#[derive(Component)]
pub struct CameraController {
    pub sensitivity: f32,
    pub distance: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            sensitivity: 0.5,
            distance: 10.0,
        }
    }
}

/// 相机旋转系统
pub fn camera_control(
    mut query: Query<&mut Transform, With<CameraController>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
) {
    if mouse_button.pressed(MouseButton::Right) {
        for mut transform in query.iter_mut() {
            for motion in mouse_motion.read() {
                let yaw = -motion.delta.x * 0.003;
                let pitch = -motion.delta.y * 0.003;

                transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(yaw));
                transform.rotate_local_x(pitch);
            }
        }
    }
}

/// 机器人键盘控制
pub fn robot_keyboard_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut ExternalForce, With<crate::models::humanoid::Torso>>,
) {
    for mut force in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if keyboard.pressed(KeyCode::Space) {
            direction.y += 1.0;
        }

        if direction.length() > 0.0 {
            force.force = direction.normalize() * 50.0;
        } else {
            force.force = Vec3::ZERO;
        }
    }
}

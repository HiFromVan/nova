//! 控制模块 - 负责机器人运动控制

use bevy::prelude::*;

mod foot_contact;
mod pd_controller;
mod walking;

pub use foot_contact::{foot_contact_detection, FootContact};
pub use pd_controller::pd_standing_control;
pub use walking::{walking_control, WalkingGait};

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

/// 相机旋转系统（鼠标右键拖动）
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

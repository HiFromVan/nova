use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// 初始化物理世界
pub fn setup_physics(mut commands: Commands) {
    // 添加光源
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // 添加相机
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    // 创建地面
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));
}

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct Humanoid;

#[derive(Component)]
pub struct Torso;

#[derive(Component)]
pub struct Leg;

/// 创建简单的人形机器人（由几个刚体组成）
pub fn spawn_simple_humanoid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let torso_mesh = meshes.add(Cuboid::new(0.4, 0.8, 0.2));
    let leg_mesh = meshes.add(Cuboid::new(0.15, 0.6, 0.15));
    let material = materials.add(Color::srgb(0.3, 0.5, 0.8));

    // 躯干
    commands.spawn((
        Humanoid,
        Torso,
        RigidBody::Dynamic,
        Collider::cuboid(0.2, 0.4, 0.1),
        Mesh3d(torso_mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));

    // 左腿
    commands.spawn((
        Humanoid,
        Leg,
        RigidBody::Dynamic,
        Collider::cuboid(0.075, 0.3, 0.075),
        Mesh3d(leg_mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(-0.15, 1.2, 0.0),
    ));

    // 右腿
    commands.spawn((
        Humanoid,
        Leg,
        RigidBody::Dynamic,
        Collider::cuboid(0.075, 0.3, 0.075),
        Mesh3d(leg_mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.15, 1.2, 0.0),
    ));
}

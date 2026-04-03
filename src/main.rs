use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod models;
mod simulation;
mod control;
mod brain_interface;

use models::humanoid;
use simulation::physics;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, physics::setup_physics)
        .add_systems(Startup, spawn_humanoid)
        .run();
}

fn spawn_humanoid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    humanoid::spawn_simple_humanoid(&mut commands, &mut meshes, &mut materials);
}

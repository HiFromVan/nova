// 临时测试文件 - 验证机器人生成
use bevy::prelude::*;

fn main() {
    println!("Testing Bevy 0.15 mesh spawning...");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, test_spawn)
        .add_systems(Update, count_entities)
        .run();
}

fn test_spawn(mut commands: Commands) {
    println!("=== Startup system running ===");
    commands.spawn((
        Transform::default(),
        Visibility::default(),
    ));
    println!("=== Entity spawned ===");
}

fn count_entities(query: Query<Entity>) {
    let count = query.iter().count();
    if count > 0 {
        println!("Total entities: {}", count);
    }
}

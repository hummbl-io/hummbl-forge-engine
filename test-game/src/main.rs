//! Minimal Bevy game — a moving circle.
//!
//! This is a test game to prove the Forge Engine pipeline end-to-end:
//! forge build → forge publish → runnable package.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Forge Test Game".to_string(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, move_circle)
        .run();
}

#[derive(Component)]
struct Player {
    velocity: Vec3,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.75, 0.5),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Player {
            velocity: Vec3::new(200.0, 150.0, 0.0),
        },
    ));
}

fn move_circle(mut query: Query<(&mut Transform, &mut Player)>, time: Res<Time>) {
    for (mut transform, mut player) in &mut query {
        let delta = time.delta_seconds();
        transform.translation += player.velocity * delta;

        // Bounce off screen edges (800x600)
        if transform.translation.x > 375.0 || transform.translation.x < -375.0 {
            player.velocity.x = -player.velocity.x;
        }
        if transform.translation.y > 275.0 || transform.translation.y < -275.0 {
            player.velocity.y = -player.velocity.y;
        }
    }
}

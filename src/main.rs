//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)// pulls in default plugin list, ECS, 2d rendering etc
        .add_systems(Startup, setup)// make the initialize() using the setup function
        .add_systems(Update, sprite_movement) // make the game loop using sprite_movement() function
        .run();
}

#[derive(Component)]
#[derive(PartialEq)]
enum Direction {
    Up,
    UpLeft,
    Left,
    Right,
    UpRight,
    Down,
    DownLeft,
    DownRight
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/pink_box.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::DownRight,
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        let delta = 150. * time.delta_seconds();

        match *logo {
            Direction::Up => transform.translation.y += delta,
            Direction::Down => transform.translation.y -= delta,
            Direction::Left => transform.translation.x -= delta,
            Direction::Right => transform.translation.x += delta,
            Direction::UpRight => {
                transform.translation.y += delta;
                transform.translation.x += delta;
            }
            Direction::UpLeft => {
                transform.translation.y += delta;
                transform.translation.x -= delta;
            }
            Direction::DownRight => {
                transform.translation.y -= delta;
                transform.translation.x += delta;
            }
            Direction::DownLeft => {
                transform.translation.y -= delta;
                transform.translation.x -= delta;
            }
        }

        // Adjust direction based on boundary checks
        if transform.translation.y > 200. {
            if *logo == Direction::UpRight {
                *logo = Direction::DownRight;
            } else if *logo == Direction::UpLeft {
                *logo = Direction::DownLeft;
            } else {
                *logo = Direction::Down;
            }
        } else if transform.translation.y < -200. {
            if *logo == Direction::DownRight {
                *logo = Direction::UpRight;
            } else if *logo == Direction::DownLeft {
                *logo = Direction::UpLeft;
            } else {
                *logo = Direction::Up;
            }
        }

        if transform.translation.x > 200. {
            if *logo == Direction::UpRight {
                *logo = Direction::UpLeft;
            } else if *logo == Direction::DownRight {
                *logo = Direction::DownLeft;
            } else {
                *logo = Direction::Left;
            }
        } else if transform.translation.x < -200. {
            if *logo == Direction::UpLeft {
                *logo = Direction::UpRight;
            } else if *logo == Direction::DownLeft {
                *logo = Direction::DownRight;
            } else {
                *logo = Direction::Right;
            }
        }
    }
}


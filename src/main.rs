//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;

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
    DownRight,
    None
}

#[derive(Resource)] // Add this line
struct Score {
    enemies_killed: u32,
}

impl Score {
    fn new() -> Self {
        Score {
            enemies_killed: 0,
        }
    }

    fn increment(&mut self) {
        self.enemies_killed += 1;
    }

    fn get_enemies_killed(&self) -> u32 {
        self.enemies_killed
    }
}

fn main() {
    App::new()
        .insert_resource(Score::new())
        .add_plugins(DefaultPlugins)// pulls in default plugin list, ECS, 2d rendering etc
        .add_systems(Startup, setup)// make the initialize() using the setup function
        .add_systems(FixedUpdate, (sprite_movement, enemy_killed, display_score).chain()) // make the game loop using sprite_movement() function
        .run();
}

// System to simulate enemy kills
fn enemy_killed(mut score: ResMut<Score>) {
    score.increment();
}

// System to display the current score
fn display_score(score: Res<Score>) {
    println!("Enemies killed: {}", score.get_enemies_killed());
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
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for (mut logo, mut transform) in &mut sprite_position {
        let delta = 150. * time.delta_seconds();

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                *logo = Direction::UpLeft;
            }
            else if   keyboard_input.pressed(KeyCode::ArrowDown) {
                *logo = Direction::DownLeft;
            }
            else {
                *logo = Direction::Left;
            }
        }
        else if keyboard_input.pressed(KeyCode::ArrowRight) {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                *logo = Direction::UpRight;
            }
            else if   keyboard_input.pressed(KeyCode::ArrowDown) {
                *logo = Direction::DownRight;
            }
            else {
                *logo = Direction::Right;
            }
        }
        else if keyboard_input.pressed(KeyCode::ArrowDown) {
            *logo = Direction::Down;
        }
        else if keyboard_input.pressed(KeyCode::ArrowUp) {
            *logo = Direction::Up;
        }
        else {
            *logo = Direction::None;
        }

        match *logo {
            Direction::None => {
                transform.translation.y = transform.translation.y;
                transform.translation.x = transform.translation.x;
            }
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
    }
}



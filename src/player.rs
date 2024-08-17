use crate::{GameTextures, BASE_SPEED, SPRITE_SCALE, SPRITE_SIZE, TIME_STEP };
use crate::components::{CollisionBox, Player, Velocity}; 
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, player_spawn_system)
            .add_systems(FixedUpdate, (player_movement_system, player_keyboard_event_system));
    }
}

fn player_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    commands.spawn((
            SpriteBundle {
                texture: game_textures.player.clone(),
                transform: Transform { 
                    translation: Vec3::new(0., SPRITE_SIZE.1 / 2. + 5., 10.),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 0.),
                    ..Default::default()
                },
                ..Default::default()
            },
    ))
        .insert(CollisionBox::new(SPRITE_SIZE.0 * SPRITE_SCALE, SPRITE_SIZE.1 * SPRITE_SCALE))
        .insert(Player::new(500))
        .insert(Velocity { x: 0., y: 0. });
}

fn player_keyboard_event_system(
    kb: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.x = if kb.pressed(KeyCode::KeyA) {
            -1.
        } else if kb.pressed(KeyCode::KeyD) {
            1.
        } else {
            0.
        };

        velocity.y = if kb.pressed(KeyCode::KeyS) {
            -1.
        } else if kb.pressed(KeyCode::KeyW) {
            1.
        } else {
            0.
        };
    }
}

fn player_movement_system(
    mut query: Query<(&Velocity, &mut Transform), With<Player>>
) {
    for (velocity, mut transform) in query.iter_mut() {
       let translation = &mut transform.translation;

       translation.x += velocity.x * TIME_STEP * BASE_SPEED;
       translation.y += velocity.y * TIME_STEP * BASE_SPEED;
    }
}
//pub fn spawn_enemies_over_time(
//    mut commands: Commands,
//    time: Res<Time>,
//    game_textures: Res<GameTextures>,
//    player_query: Query<&Transform, With<Player>>, // Query to get the player's Transform
//    mut spawn_timer: ResMut<EnemySpawnTimer>,
//) {
//    // Update the timer
//    spawn_timer.timer.tick(time.delta());
//
//    // If the timer has finished and we haven't spawned all enemies
//    if spawn_timer.timer.finished() && spawn_timer.enemies_spawned < spawn_timer.total_enemies {
//        if let Ok(player_transform) = player_query.get_single() {
//            let player_position = player_transform.translation;
//
//            // Generate a random angle between 0 and 2π radians (full circle)
//            let mut rng = rand::thread_rng();
//            let angle = rng.gen_range(0.0..(2.0 * PI));
//
//            // Calculate the x and y position based on the angle and radius
//            let x = player_position.x + spawn_timer.spawn_radius * angle.cos();
//            let y = player_position.y + spawn_timer.spawn_radius * angle.sin();
//
//            // Spawn the enemy entity at the calculated position
//            commands.spawn((
//                    SpriteBundle {
//                        texture: game_textures.enemy.clone(), 
//                        transform: Transform { 
//                            translation: Vec3::new(0., SPRITE_SIZE.1 / 2. + 5., 10.),
//                            scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 0.),
//                            ..Default::default()
//                        },
//                        ..Default::default()
//
//                    },
//            ))
//                .insert(CollisionBox::new(50.0, 50.0)) // Add collision box
//                .insert(Tag { name: format!("Enemy{}", spawn_timer.enemies_spawned) }) // Tag with a unique name
//                .insert(MovementSpeed(50.0)) // Set movement speed
//                .insert(DirectionComponent { direction: Direction::None }); // Set initial direction
//
//            // Increment the count of spawned enemies
//            spawn_timer.enemies_spawned += 1;
//        }
//    }
//}

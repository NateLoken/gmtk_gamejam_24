use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::Rng;

use crate::{components::{Boss, Collider, Enemy, Health, Invulnerability, Player, Velocity}, EnemyCount, GameTextures, ENEMY_SPEED, MAX_ENEMIES, PLAYER_RADIUS, SPRITE_SCALE, SPRITE_SIZE, TIME_STEP };
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
   fn build(&self, app: &mut App) {
       app.add_systems(PostStartup, spawn_boss)
           .add_systems(Update, enemy_spawn_system.run_if(on_timer(Duration::from_secs(1))))
           .add_systems(FixedUpdate, (player_tracking_system, enemy_movement_system));
   } 
}

fn enemy_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut enemy_count: ResMut<EnemyCount>,
    player_query: Query<&Transform, With<Player>>
) {
    if enemy_count.0 < MAX_ENEMIES {
        if let Ok(player_transform) = player_query.get_single() {
            let player_position = player_transform.translation;

            let mut rng = rand::thread_rng();
            let angle = rng.gen_range(0.0..(2.0 * PI));

            let x = player_position.x + PLAYER_RADIUS * angle.cos();
            let y = player_position.y + PLAYER_RADIUS * angle.sin();

            commands.spawn((
                    SpriteBundle {
                        texture: game_textures.enemy.clone(),
                        transform: Transform {
                            translation: Vec3::new(x, y, 10.),
                            scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 0.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Health {
                        hp: 1,
                    },
                    Collider::new(Vec2::splat(SPRITE_SIZE.0 * SPRITE_SCALE)),
                    Enemy,
                    Velocity {
                        x: 0.,
                        y: 0.,
                    },
            ));
            enemy_count.0 += 1;
        }
    }
}

fn spawn_boss(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    player_query: Query<&Transform, With<Player>>
) {
    if let Ok(player_transform) = player_query.get_single() {
        println!("Spawning Boss");
        let player_position = player_transform.translation;

        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..(2. * PI));

        let x = player_position.x  * angle.cos();
        let y = player_position.y  * angle.sin();

        commands.spawn((
            SpriteBundle {
                texture: game_textures.boss.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 10.),
                    scale: Vec3::new(SPRITE_SCALE * 2., SPRITE_SCALE * 2., 0.),
                    ..Default::default()
                },
                ..Default::default()
            },
            Collider::new(Vec2::splat(SPRITE_SIZE.0 * SPRITE_SCALE)),
            Enemy,
            Boss,
            ));
    }
}

fn player_tracking_system(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&mut Velocity, &Transform),With<Enemy>>,
) { 
    if let Ok(player_transform) = player_query.get_single() {
        for (mut velocity, enemy_transform) in enemy_query.iter_mut() {
            let direction_vector = (player_transform.translation - enemy_transform.translation).normalize();
            velocity.x = direction_vector.x;
            velocity.y = direction_vector.y;
        }
    }
}

fn enemy_movement_system(
    mut query: Query<(&Velocity, &mut Transform), With<Enemy>>
) {
    for (velocity, mut transform) in query.iter_mut() {
       let translation = &mut transform.translation;

       translation.x += velocity.x * TIME_STEP * ENEMY_SPEED;
       translation.y += velocity.y * TIME_STEP * ENEMY_SPEED;
    }
}

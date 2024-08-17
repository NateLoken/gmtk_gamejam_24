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
            if kb.pressed(KeyCode::KeyW) || kb.pressed(KeyCode::KeyS) {
                -1. / 2.
            } else {
                -1.
            }
        } else if kb.pressed(KeyCode::KeyD) {
            if kb.pressed(KeyCode::KeyW) || kb.pressed(KeyCode::KeyS){
                1. / 2.
            } else {
                1.
            }
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


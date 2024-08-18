use crate::{GameTextures, MouseCoords, BASE_SPEED, SPRITE_SCALE, SPRITE_SIZE, TIME_STEP };
use crate::components::{Ability, CollisionBox, Cooldowns, Invulnerability, Lifetime, Line, Player, Velocity}; 
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, player_spawn_system)
            .add_systems(FixedUpdate, (
                    player_movement_system, 
                    player_keyboard_event_system,
                    ability_system,));
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
        .insert(Player { health: 500 })
        .insert(Velocity { x: 0., y: 0. })
        .insert(Cooldowns::new());  // Initialize cooldowns for abilities)
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

fn ability_system(
    commands: Commands,
    kb: Res<ButtonInput<KeyCode>>,
    mut cooldown_query: Query<&mut Cooldowns>,
    mouse_coords: Res<MouseCoords>,
    mut player_query: Query<(Entity, &mut Transform), With<Player>>,
    game_textures: Res<GameTextures>
) {
    if let Ok(mut cooldowns) = cooldown_query.get_single_mut() {
        if kb.just_pressed(KeyCode::KeyF) {
            if cooldowns.is_ready(Ability::Dash) {
                dash_attack(
                    commands,
                    player_query,
                    mouse_coords,
                    game_textures);
                cooldowns.reset(Ability::Dash);
            } else {
                println!("Dash is on cooldown!");
            }
        }
    }
}

pub fn dash_attack(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Player>>,
    mouse_coords: Res<MouseCoords>,
    game_textures: Res<GameTextures>,
) {
    if let Ok((player_entity, mut transform)) = query.get_single_mut() {
        let player_position = Vec2::new(transform.translation.x, transform.translation.y);
        let mouse_position = Vec2::new(mouse_coords.x, mouse_coords.y);
        let direction = mouse_position - player_position;
        let length = direction.length();

        let midpoint = player_position + direction / 2.;

        let angle = direction.y.atan2(direction.x);

        commands.spawn(SpriteBundle {
            texture: game_textures.dash.clone(),
            transform: Transform {
                translation: Vec3::new(midpoint.x, midpoint.y, 0.),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::new(length, SPRITE_SCALE, 0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Line)
        .insert(CollisionBox::new(length, SPRITE_SIZE.0))
        .insert(Lifetime {
            timer: Timer::from_seconds(0.1, TimerMode::Once)
        });

        commands.entity(player_entity).insert(Invulnerability {
            timer: Timer::from_seconds(0.5, TimerMode::Once)
        });

        transform.translation.x = mouse_position.x;
        transform.translation.y = mouse_position.y;
    }
}

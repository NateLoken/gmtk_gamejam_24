use std::f32::consts::PI;

use crate::{GameTextures, MouseCoords, BASE_SPEED, SPRITE_SCALE, SPRITE_SIZE, TIME_STEP };
use crate::components::{Ability, CollisionBox, Cooldowns, GameState, Invulnerability, Lifetime, Line, Player, PointMarker, Points, Velocity}; 
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, player_spawn_system)
            .add_systems(FixedUpdate, (
                    player_movement_system, 
                    player_keyboard_event_system,
                    ability_system,).run_if(in_state(GameState::Running)));
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
                    scale: Vec3::new(SPRITE_SCALE/2.5, SPRITE_SCALE/2.5, 0.),
                    ..Default::default()
                },
                ..Default::default()
            },
    ))
        .insert(CollisionBox::new(SPRITE_SIZE.0 * SPRITE_SCALE, SPRITE_SIZE.1 * SPRITE_SCALE))
        .insert(Player { health: 500, x:0.0, y:0.0 })
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
    player_query: Query<(Entity, &mut Transform), With<Player>>,
    game_textures: Res<GameTextures>,
    points: ResMut<Points>
) {
    if let Ok(mut cooldowns) = cooldown_query.get_single_mut() {
        if kb.just_pressed(KeyCode::KeyE) {
            if cooldowns.is_ready(Ability::Ranged) {
                ranged_attack(
                    commands,
                    player_query,
                    mouse_coords,
                    game_textures);
                cooldowns.reset(Ability::Ranged);
            } else {
                println!("Ranged ability on cooldown!");
            }
        } else if kb.just_pressed(KeyCode::KeyF) {
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
        } else if kb.just_pressed(KeyCode::KeyQ) {
            if cooldowns.is_ready(Ability::Attack) {
                melee_attack(
                    commands,
                    player_query,
                    mouse_coords,
                    game_textures,
                    points);
                cooldowns.reset(Ability::Attack);
            } else {
                println!("Arc ability is on cooldown!");
            }
        } else if kb.just_pressed(KeyCode::KeyT) {
            if cooldowns.is_ready(Ability::Aoe) {
                aoe_attack(
                    commands, 
                    player_query, 
                    game_textures, 
                    points);
                cooldowns.reset(Ability::Aoe);
            } else {
                println!("AOE is on cooldown!");
            }
        }
    }
}

fn ranged_attack(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform), With<Player>>,
    mouse_coords: Res<MouseCoords>,
    game_textures: Res<GameTextures>,
) {
    if let Ok((_, transform)) = query.get_single() {
        let player_position = Vec2::new(transform.translation.x, transform.translation.y);
        let mouse_position = Vec2::new(mouse_coords.x, mouse_coords.y);

         // Calculate direction from player to mouse
        let direction = (mouse_position - player_position).normalize();
        
        // Set the desired line length
        let line_length = 1100.0;

        // Calculate the endpoint of the line
        let end_point = player_position + direction * line_length;

        // Calculate the midpoint of the line for positioning the sprite
        let midpoint = (player_position + end_point) / 2.0;

        // Calculate the angle for proper rotation
        let angle = direction.y.atan2(direction.x);

        commands.spawn(
            SpriteBundle {
                texture: game_textures.dash.clone(),
                transform: Transform {
                    translation: Vec3::new(midpoint.x, midpoint.y, 0.),
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::new(1100.0, SPRITE_SCALE, 0.),
                    ..Default::default()
                },
                ..Default::default()
            }
        )
            .insert(Line)
            .insert(CollisionBox::new(1100.0, 100.0))
            .insert(Lifetime {
                timer: Timer::from_seconds(0.1, TimerMode::Once)
            });
    }
}

fn dash_attack(
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

fn melee_attack(
    mut commands: Commands,
    player_query: Query<(Entity, &mut Transform), With<Player>>,
    mouse_coords: Res<MouseCoords>,
    game_textures: Res<GameTextures>,
    mut points: ResMut<Points>,
) {
    if let Ok((_, transform)) = player_query.get_single() {
        let player_position = Vec2::new(transform.translation.x, transform.translation.y);
        let mouse_position = Vec2::new(mouse_coords.x, mouse_coords.y);

        let direction = (mouse_position - player_position).normalize();
        let start_angle = direction.y.atan2(direction.x);

        let max_radius = 250.0; // Max radius for the arc
        let theta = 0.0725; // Smaller theta for finer increments
        let arc_span = PI / 2.0; // 90 degrees in radians
        let radius_step = 10.0; // Distance between each concentric arc


        let arc_segments = (arc_span / theta) as i32;

        points.0.clear();

        for radius in (radius_step as i32..=max_radius as i32).step_by(radius_step as usize) {
            for i in 0..=arc_segments {
                let angle = start_angle - (arc_span / 2.) + i as f32 * theta;
                let arc_point = Vec2::new(
                    player_position.x + radius as f32 * angle.cos(),
                    player_position.y + radius as f32 * angle.sin(),
                );

                points.0.push(arc_point);

                commands.spawn(
                    SpriteBundle {
                        texture: game_textures.dash.clone(),
                        transform: Transform {
                            translation: Vec3::new(arc_point.x, arc_point.y, 0.),
                            scale: Vec3::new(5., 5., 0.),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PointMarker)
                    .insert(Lifetime {
                        timer: Timer::from_seconds(0.1, TimerMode::Once),
                    });
            }
        }
    }
}


fn aoe_attack(
    mut commands: Commands,
    player_query: Query<(Entity, &mut Transform), With<Player>>,
    game_textures: Res<GameTextures>,
    mut points: ResMut<Points>
) {
    if let Ok((_, transform)) = player_query.get_single() {
        let player_position = Vec2::new(transform.translation.x, transform.translation.y);

        let max_radius = 300.;
        let theta = 0.0725;
        let total_angle = 2.0 * PI;
        let radius_step = 10.;
        let arc_segments = (total_angle / theta) as i32;

        points.0.clear();

        for radius in(0..=max_radius as i32).step_by(radius_step as usize) {
            for i in 0..=arc_segments {
                let angle = i as f32 * theta;
                let circle_point = Vec2::new(
                    player_position.x + radius as f32 * angle.cos(), 
                    player_position.y + radius as f32 * angle.sin(),
                    );

                points.0.push(circle_point);

                commands.spawn(
                    SpriteBundle {
                        texture: game_textures.dash.clone(),
                        transform: Transform {
                            translation: Vec3::new(circle_point.x, circle_point.y, 0.),
                            scale: Vec3::new(5., 5., 0.),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PointMarker)
                    .insert(Lifetime {
                        timer: Timer::from_seconds(0.1, TimerMode::Once),
                    });
            }
        }
    }
}

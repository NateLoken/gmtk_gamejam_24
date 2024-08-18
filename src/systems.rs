use bevy::prelude::*;
use crate::components::{Cooldowns, CollisionBox, Invulnerability, Lifetime, Line, PointMarker, Points, Player};
use crate::events::{CollisionEvent, Score};
use std::f32::consts::PI;

use crate::{EnemyCount, GameTextures, MouseCoords, ENEMY_SPRITE, LINE_SPRITE, PLAYER_SPRITE};
// Systems Implementation

pub fn camera_follow_player(
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,             // Query to get the player's position
        Query<&mut Transform, With<Camera2d>>,       // Query to get the camera's Transform
    )>,
    window_query: Query<&Window>,                    // Query to get the window
) {
    // First, get the player's Transform
    let player_position = {
        if let Ok(player_transform) = param_set.p0().get_single() {
            Some(player_transform.translation)
        } else {
            None
        }
    };

    // If we have the player's position, continue
    if let Some(player_position) = player_position {
        // Then, get the window dimensions
        if let Ok(window) = window_query.get_single() {
            // Now we can safely get the camera's Transform
            if let Ok(mut camera_transform) = param_set.p1().get_single_mut() {
                let half_width = window.width() / 2.0;
                let half_height = window.height() / 2.0;

                // Calculate world bounds, ensuring min < max
                let min_x = -500.0 + half_width;
                let max_x = 500.0 - half_width;
                let min_y = -500.0 + half_height;
                let max_y = 500.0 - half_height;

                // Ensure bounds are valid (min should be less than max)
                if min_x < max_x && min_y < max_y {
                    // Calculate camera position with clamping
                    let camera_x = player_position.x.clamp(min_x, max_x);
                    let camera_y = player_position.y.clamp(min_y, max_y);

                    camera_transform.translation.x = camera_x;
                    camera_transform.translation.y = camera_y;
                } else {
                    // Handle the edge case where bounds are not valid (e.g., world is smaller than window)
                    camera_transform.translation.x = player_position.x;
                    camera_transform.translation.y = player_position.y;
                }
            }
        }
    }
}


pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut score: ResMut<Score>, // Access the Score resource
) {
    let mut entities_to_despawn = Vec::new(); // Collect entities to despawn after the loop
    collision_events
        .read() // Use par_read to access events in a parallel-safe manner
        .for_each(|CollisionEvent(entity)| {
            if keyboard_input.pressed(KeyCode::KeyR) {
                entities_to_despawn.push(*entity); // Mark the entity for despawning + needs to be this way to avoid segfault
                enemy_killed(&mut score);
            }
        });
    for entity in entities_to_despawn {
        if commands.get_entity(entity).is_some() { //make sure it exists
            commands.entity(entity).despawn(); //despawn all
        }
    }
}

pub fn enemy_killed(score: &mut ResMut<Score>) {
    score.increment();
    println!("Score: {}", score.get_enemies_killed());
}

pub fn display_score(_score: Res<Score>) {
    //println!("Enemies killed: {}", score.get_enemies_killed());
}

pub fn check_collisions(
    mut player_query: Query<(&mut Player, &CollisionBox, &Transform), Without<Invulnerability>>,
    other_entities_query: Query<(Entity, &Transform, &CollisionBox), Without<Player>>,
    points_query: Query<(Entity, &Transform), With<PointMarker>>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    line_query: Query<(Entity, &Transform, &CollisionBox), With<Line>>,
) {
    for (enemy_entity, transform, bounding_box) in other_entities_query.iter() {
        let enemy_min_x = transform.translation.x - bounding_box.width / 2.0;
        let enemy_max_x = transform.translation.x + bounding_box.width / 2.0;
        let enemy_min_y = transform.translation.y - bounding_box.height / 2.0;
        let enemy_max_y = transform.translation.y + bounding_box.height / 2.0;

        for (point_entity, point_transform) in points_query.iter() {
            let point = Vec2::new(point_transform.translation.x, point_transform.translation.y);

            if point.x > enemy_min_x
                && point.x < enemy_max_x
                && point.y > enemy_min_y
                && point.y < enemy_max_y
            {
                // Call the kill_enemy function
                score.increment();

                // Despawn the enemy
                commands.entity(enemy_entity).despawn();
                break;
            }
        }
    }
    for (enemy_entity, enemy_box, enemy_transform) in other_entities_query.iter() {
        let enemy_min_x = enemy_box.translation.x - enemy_transform.width / 2.0;
        let enemy_max_x = enemy_box.translation.x + enemy_transform.width / 2.0;
        let enemy_min_y = enemy_box.translation.y - enemy_transform.height / 2.0;
        let enemy_max_y = enemy_box.translation.y + enemy_transform.height / 2.0;

        for (line_entity, line_box, line_transform) in line_query.iter() {
            let line_min_x = line_box.translation.x - line_transform.width / 2.0;
            let line_max_x = line_box.translation.x + line_transform.width / 2.0;
            let line_min_y = line_box.translation.y - line_transform.height / 2.0;
            let line_max_y = line_box.translation.y + line_transform.height / 2.0;

            // Check for collision
            if line_max_x > enemy_min_x
                && line_min_x < enemy_max_x
                && line_max_y > enemy_min_y
                && line_min_y < enemy_max_y
            {
                // Call the kill_enemy function
                score.increment();

                // Despawn the enemy
                commands.entity(enemy_entity).despawn();

                // Despawn the line after it collides with an enemy
                //commands.entity(line_entity).despawn();
                break;
            }
        }
    }
    for (mut player, player_box, player_transform) in player_query.iter_mut() {
        for (enemy_entity, enemy_transform, enemy_box) in other_entities_query.iter() {
            let enemy_min_x = enemy_transform.translation.x - enemy_box.width / 2.0;
            let enemy_max_x = enemy_transform.translation.x + enemy_box.width / 2.0;
            let enemy_min_y = enemy_transform.translation.y - enemy_box.height / 2.0;
            let enemy_max_y = enemy_transform.translation.y + enemy_box.height / 2.0;

            let player_min_x = player_transform.translation.x - player_box.width / 2.0;
            let player_max_x = player_transform.translation.x + player_box.width / 2.0;
            let player_min_y = player_transform.translation.y - player_box.height / 2.0;
            let player_max_y = player_transform.translation.y + player_box.height / 2.0;

            if player_max_x > enemy_min_x
                && player_min_x < enemy_max_x
                && player_max_y > enemy_min_y
                && player_min_y < enemy_max_y
            {
                // Handle collision, but only if player is not invulnerable
                player.take_damage(10);
            }
        }
    }
}

// System to update the MousePosition resource whenever the mouse moves
pub fn update_mouse_position(
    q_windows: Query<&Window>,
    mut mouse_position: ResMut<MouseCoords>,
    camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera2d>>,
) {
    let window = q_windows.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok((camera_transform, projection)) = camera_query.get_single() {
            // Convert the cursor position to NDC (Normalized Device Coordinates)
            let window_size = Vec2::new(window.width(), window.height());
            let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;

            // Use the orthographic projection's area to convert NDC to world coordinates
            let world_position = camera_transform.translation()
                + Vec3::new(
                    ndc.x * projection.area.width() / 2.0,
                    -ndc.y * projection.area.height() / 2.0,
                    0.0,
                );

            mouse_position.x = world_position.x;
            mouse_position.y = world_position.y;

            //println!("Mouse Position in World: ({}, {})", mouse_position.x, mouse_position.y);
        }
    }
}

pub fn manage_invulnerability(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Invulnerability)>,
) {
    for (entity, mut invulnerability) in query.iter_mut() {
        invulnerability.timer.tick(time.delta());
        if invulnerability.timer.finished() {
            commands.entity(entity).remove::<Invulnerability>();
        }
    }
}

pub fn update_cooldowns(
    time: Res<Time>,
    mut query: Query<&mut Cooldowns>,
) {
    for mut cooldowns in query.iter_mut() {
        for timer in cooldowns.cooldowns.values_mut() {
            timer.tick(time.delta());
        }
    }
}

pub fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.finished() {
            commands.entity(entity).despawn();  // This command is deferred and will execute later
        }
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(
        TextBundle::from_section(
            "WASD to Move around, Q to Melee, E for Ranged, T for AoE, F to Dash",
            TextStyle {
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        }),
    );

    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        dash: asset_server.load(LINE_SPRITE),
    };

    let enemy_count = EnemyCount(0);

    let mouse_coords = MouseCoords {
        x: 0.,
        y: 0.,
    };

    commands.insert_resource(game_textures);
    commands.insert_resource(enemy_count);
    commands.insert_resource(Score::new());
    commands.insert_resource(mouse_coords);
    commands.insert_resource(Points::default());
}

use bevy::prelude::*;
use crate::components::{Cooldowns, Health, Invulnerability, Lifetime, Player, Points};
use crate::events::Score;

use crate::{EnemyCount, GameTextures, MouseCoords, BOSS_SPRITE, ENEMY_SPRITE, LINE_SPRITE, PLAYER_SPRITE};
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

pub fn clean_dead(
    mut commands: Commands,
    query: Query<(Entity, &Health)>,
) {
   for (entity_id, entity_health) in query.iter() {
       if entity_health.hp <=0 {
           commands.entity(entity_id).despawn();
       }
   }
}

pub fn display_score(_score: Res<Score>) {
    //println!("Enemies killed: {}", score.get_enemies_killed());
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
        boss: asset_server.load(BOSS_SPRITE),
        line: asset_server.load(LINE_SPRITE),
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

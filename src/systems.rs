use bevy::prelude::*;
use crate::components::{ CollisionBox, Player,};
use crate::events::{CollisionEvent, Score};
use crate::{EnemyCount, GameTextures, ENEMY_SPRITE, PLAYER_SPRITE};
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
            if keyboard_input.pressed(KeyCode::KeyE) {
                entities_to_despawn.push(*entity); // Mark the entity for despawning + needs to be this way to avoid segfault
                enemy_killed(&mut score);
            }
            // Example: increment the counter
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
    mut player_query: Query<(&Transform, &CollisionBox, &mut Player)>,
    other_entities_query: Query<(Entity, &Transform, &CollisionBox), Without<Player>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    for (player_transform, player_collision_box, mut player) in player_query.iter_mut() {
        for (entity, other_transform, other_collision_box) in other_entities_query.iter() {
            if player_collision_box.intersects(player_transform, other_collision_box, other_transform) {
                collision_events.send(CollisionEvent(entity));
                player.take_damage(10);
            }
        }
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(
        TextBundle::from_section(
            "WASD to Move around, E to kill",
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
    };

    let enemy_count = EnemyCount(0);
    commands.insert_resource(game_textures);
    commands.insert_resource(enemy_count);
}

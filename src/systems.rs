use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use crate::components::{
    Ability, Bigfoot, BigfootState, Collider, CooldownUi, Cooldowns, GameState, GameTimer,
    GameTimerText, Health, HealthText, Invulnerability, Lifetime, Map, MapGrid, Player, Points,
    Resettable, Score, ScoreText,
};
use crate::{
    EnemySpawnRate, GameTextures, MouseCoords, ENEMY_SPRITE, LINE_SPRITE, MAP_SPIRITE,
    PLAYER_SPRITE,
};
use rand::Rng;
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
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &Health, Option<&Player>)>,
) {
    let mut trigger_game_over = false;

    for (entity, health, maybe_player) in query.iter_mut() {
        if health.hp <= 0 {
            if maybe_player.is_some() {
                trigger_game_over = true;
            }
            commands.entity(entity).despawn_recursive();
        }
    }

    if trigger_game_over {
        death_sound(&asset_server, &mut commands);
        next_state.set(GameState::GameOver);
    }
}

//pub fn enemy_killed(score: &mut ResMut<Score>, mut player: &mut Player, cooldowns_query: &mut Query<&mut Cooldowns>,) {
//    score.increment();
//    player.heal(1);
//    println!("Score: {}", score.get_enemies_killed());
//     // Apply cooldown reduction to all abilities
//     // Apply cooldown reduction to all abilities
//     for mut cooldowns in cooldowns_query.iter_mut() {
//        for timer in cooldowns.cooldowns.values_mut() {
//            // Calculate the new elapsed time, ensuring it doesn't go below zero
//            let elapsed_time = timer.elapsed_secs() + 0.05;
//
//            // Manually set the timer's tick to the new elapsed time
//            timer.set_elapsed(Duration::from_secs_f32(elapsed_time.min(timer.duration().as_secs_f32())));
//        }
//    }
//}

pub fn spawn_bigfoot(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
    existing_bigfoot: Query<Entity, With<Bigfoot>>,
) {
    if existing_bigfoot.iter().next().is_some() {
        return;
    }

    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation;

        commands
            .spawn((
                SpriteBundle {
                    texture: asset_server.load("foot.png"),
                    transform: Transform {
                        translation: Vec3::new(100.0, player_position.y, 1.0),
                        scale: Vec3::new(0.7, 0.7, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Bigfoot {
                    timer: Timer::from_seconds(2.5, TimerMode::Once),
                    state: BigfootState::Invulnerable,
                    health: 5,
                    x: player_position.x,
                    y: player_position.y,
                    airTexture: asset_server.load("foot.png"),
                    groundTexture: asset_server.load("foot_ground.png"),
                },
                Collider::new(Vec2::new(256.0, 256.0)),
                Resettable,
            ));
    }
}


pub fn update_bigfoot(
    mut query: Query<(Entity, &mut Bigfoot, &mut Sprite, &mut Transform, &mut Handle<Image>), Without<Player>>,
    mut player_query: Query<(&mut Transform, Option<&mut Invulnerability>), With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (_entity, mut bigfoot, mut sprite, mut transform, mut texture) in query.iter_mut() {
        // Update Bigfoot's timer
        bigfoot.timer.tick(time.delta());

        if bigfoot.timer.just_finished() {
            match bigfoot.state {
                BigfootState::Invulnerable => {
                    // Switch to the stomp phase
                    bigfoot.state = BigfootState::Solid;

                    // Make Bigfoot fully opaque and solid
                    sprite.color.set_alpha(1.0);

                    // Set the timer for the stomp phase
                    bigfoot.timer = Timer::from_seconds(5.0, TimerMode::Once);

                    // Change the texture based on the state
                    cycle_texture(&mut texture, &bigfoot);
                    stomp_sound(&asset_server, &mut commands);

                    if let Ok((mut player_transform, _invulnerability_option)) = player_query.get_single_mut() {
                        let player_position = Vec3 { x: player_transform.translation.x, y: player_transform.translation.y, z: 1.0 };
                        let bigfoot_position = Vec3 { x: bigfoot.x, y: bigfoot.y, z: 1.0 };

                        // If the player is within the 250 radius, apply damage
                        let distance = player_position.distance(bigfoot_position);
                        if distance <= 175.0 {
                            //if let Some(ref mut invulnerability) = invulnerability_option {
                            //    player.take_damage(
                            //        500, // Damage amount
                            //        entity,
                            //        &mut commands,
                            //        Some(invulnerability), // Pass the mutable reference
                            //        &mut state,
                            //        &asset_server
                            //    );
                            //} else {
                            //    player.take_damage(
                            //        500, // Damage amount
                            //        entity,
                            //        &mut commands,
                            //        None, 
                            //        &mut state,
                            //        &asset_server
                            //    );
                            //}
                        }
                    }
                }
                BigfootState::Solid => {
                    // Bigfoot has finished stomping, reset its state and move it to the player's position

                    // Get the player's position
                    if let Ok((player_transform, _invulnerability_option)) = player_query.get_single() {
                        // Move Bigfoot to the player_velocity's position
                        bigfoot.x = player_transform.translation.x;
                        bigfoot.y = player_transform.translation.y;

                        // Update the transform of Bigfoot to match the new position
                        transform.translation.x = bigfoot.x;
                        transform.translation.y = bigfoot.y;

                        // Reset Bigfoot's state to Invulnerable and restart the timer
                        bigfoot.state = BigfootState::Invulnerable;
                        bigfoot.timer = Timer::from_seconds(2.5, TimerMode::Once);

                        // Make Bigfoot semi-transparent again
                        sprite.color.set_alpha(0.5);
                        cycle_texture(&mut texture, &bigfoot);
                    }
                }
                BigfootState::Cleanup => todo!(),
            }
        } else if bigfoot.state == BigfootState::Invulnerable {
            // While Bigfoot is invulnerable, make it semi-transparent
            sprite.color.set_alpha(0.5);
        }
    }
}

fn cycle_texture(
    texture: &mut Handle<Image>,
    bigfoot: &Bigfoot,
) {
    if *texture == bigfoot.airTexture {
        *texture = bigfoot.groundTexture.clone();
    } else {
        *texture = bigfoot.airTexture.clone();
    }
}

pub fn update_bigfoot_position(
    mut bigfoot_query: Query<(&mut Bigfoot, &Transform)>,
) {
    for (mut bigfoot, transform) in bigfoot_query.iter_mut() {
        bigfoot.x = transform.translation.x;
        bigfoot.y = transform.translation.y;
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
        println!(
            "Invulnerability timer ticking for entity {:?}, remaining: {:.2}",
            entity,
            invulnerability.timer.remaining_secs()
        );
        if invulnerability.timer.finished() {
            println!("Invulnerability expired for entity {:?}", entity);
            commands.entity(entity).remove::<Invulnerability>(); // Remove the component when the timer is done
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

pub fn update_cooldowns_ui(
    mut cooldowns_query: Query<&mut Cooldowns>,
    mut text_query: Query<&mut Text, With<CooldownUi>>,
) {
    if let Ok(cooldowns) = cooldowns_query.get_single_mut() {
        // Update the UI text for each ability
        for (i, mut text) in text_query.iter_mut().enumerate() {
            let ability_text = match i {
                0 => format_cooldown_text("Attack", cooldowns.get_cooldown(Ability::Attack)),
                1 => format_cooldown_text("Ranged", cooldowns.get_cooldown(Ability::Ranged)),
                2 => format_cooldown_text("Dash", cooldowns.get_cooldown(Ability::Dash)),
                3 => format_cooldown_text("Aoe", cooldowns.get_cooldown(Ability::Aoe)),
                _ => "Unknown Ability".to_string(),
            };

            text.sections[0].value = ability_text;
        }
    }
}

fn format_cooldown_text(name: &str, cooldown: Option<f32>) -> String {
    let display_time = cooldown.unwrap_or(0.0);
    let display_time = if display_time > 0.0 {
        display_time
    } else {
        0.0
    };
    format!("{}: {:.1}s", name, display_time)
}

pub fn update_ui_text(
    player_query: Query<&Health, With<Player>>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    mut text_query: Query<(&mut Text, Option<&HealthText>, Option<&ScoreText>, Option<&GameTimerText>)>,
) {
    if let Ok(player_health) = player_query.get_single() {
        for (mut text, health_text, score_text, timer_text) in text_query.iter_mut() {
            if health_text.is_some() {
                text.sections[0].value = format!("Health: {}", player_health.hp);
            } else if score_text.is_some() {
                text.sections[0].value = format!("Score: {}", score.get_enemies_killed());
            }else if timer_text.is_some() {
                text.sections[0].value = format!("Time: {}", f32::trunc(timer.0 * 100.0)/ 100.)
            }
        }
    }
}

const MAP_WIDTH: f32 = 2672.0*4.0;
const MAP_HEIGHT: f32 = 1312.0*4.0;
const MAP_SPAWN_THRESHOLD: f32 = 500.0; // Adjust as necessary


pub fn check_and_spawn_map(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut map_query: Query<(Entity, &Transform), With<Map>>,
    mut map_grid: ResMut<MapGrid>,
    game_textures: Res<GameTextures>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation;

        // Calculate the player's grid position
        let player_grid_x = (player_pos.x / MAP_WIDTH).round() as i32;
        let player_grid_y = (player_pos.y / MAP_HEIGHT).round() as i32;

        // Check the surrounding 8 grid positions and spawn maps if necessary
        for dx in -1..=1 {
            for dy in -1..=1 {
                let grid_x = player_grid_x + dx;
                let grid_y = player_grid_y + dy;

                if !map_grid.positions.contains(&(grid_x, grid_y)) {
                    // Spawn a new map at this grid position
                    commands.spawn((
                            SpriteBundle {
                                texture: game_textures.map.clone(),
                                transform: Transform {
                                    translation: Vec3::new(grid_x as f32 * MAP_WIDTH, grid_y as f32 * MAP_HEIGHT, 0.0),
                                    scale: Vec3::new(4.0, 4.0, 0.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            Map,
                    ));
                    // Add the new map's position to the grid
                    map_grid.positions.insert((grid_x, grid_y));
                }
            }
        }

        // Now, clean up maps that are not adjacent to the player's position
        for (entity, map_transform) in map_query.iter_mut() {
            let map_pos = map_transform.translation;
            let map_grid_x = (map_pos.x / MAP_WIDTH).round() as i32;
            let map_grid_y = (map_pos.y / MAP_HEIGHT).round() as i32;

            // Check if the map is within a 3x3 grid around the player
            if (map_grid_x - player_grid_x).abs() > 1 || (map_grid_y - player_grid_y).abs() > 1 {
                // Despawn maps that are outside this grid
                commands.entity(entity).despawn();
                map_grid.positions.remove(&(map_grid_x, map_grid_y));
            }
        }
    }
}

pub fn death_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/death.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.3),
            ..Default::default()
        }
    });
}

pub fn play_empty_swing(
    asset_server: Res<AssetServer>,
    commands: &mut Commands
) {
    let sound1 = "sfx/swing1.ogg";
    let sound2 = "sfx/swing2.ogg";
    let sound3 = "sfx/swing3.ogg";

    // Collect the sounds into a vector
    let sounds = vec![sound1, sound2, sound3];

    // Generate a random index to pick a sound
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..sounds.len());

    // Select the sound based on the random index
    let selected_sound = sounds[random_index];
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load(selected_sound),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(1.0),
            ..Default::default()
        }
    });
}

pub fn play_hit_swing(
    asset_server: & Res<AssetServer>,
    commands: &mut Commands
) {
    let sound1 = "sfx/hit1.ogg";
    let sound2 = "sfx/hit2.ogg";
    let sound3 = "sfx/hit3.ogg";

    // Collect the sounds into a vector
    let sounds = vec![sound1, sound2, sound3];

    // Generate a random index to pick a sound
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..sounds.len());

    // Select the sound based on the random index
    let selected_sound = sounds[random_index];
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load(selected_sound),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(1.0),
            ..Default::default()
        }
    });
}

pub fn bone_hit(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/bone.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.15),
            ..Default::default()
        }
    });
}

pub fn dash_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/dash.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(2.75),
            ..Default::default()
        }
    });
}

pub fn reset_game(
    mut commands: Commands,
    resettable: Query<Entity, With<Resettable>>,
    mut cooldowns_query: Query<&mut Cooldowns>,
    mut score: ResMut<Score>,
    mut game_timer: ResMut<GameTimer>,
    mut points: ResMut<Points>,
    mut map_grid: ResMut<MapGrid>,
    mut enemy_spawn_rate: ResMut<EnemySpawnRate>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for entity in resettable.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for mut cooldowns in cooldowns_query.iter_mut() {
        cooldowns.reset_all();
    }

    score.reset();
    game_timer.0 = 0.0;
    points.0.clear();
    map_grid.positions.clear();
    enemy_spawn_rate.0 = 2.0;

    next_state.set(GameState::Running);
}

pub fn aoe_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/aoe.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.8),
            ..Default::default()
        }
    });
}
pub fn update_timer(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    _query: Query<&mut Text, With<GameTimerText>>,
) {
    // Accumulate time
    timer.0 += time.delta_seconds();

    // Update the text with the accumulated time
    // for mut text in query.iter_mut() {
    //     text.sections[0].value = format!("Time: {:.1}", timer.0);
    // }
}

pub fn stomp_sound(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/stomp.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.6),
            ..Default::default()
        }
    });
}

pub fn ranged_sound(
    asset_server: &mut Res<AssetServer>,
    commands: &mut Commands
) {
    // Create an entity dedicated to playing our background music
    let _ = &mut commands.spawn(AudioBundle {
        source: asset_server.load("./sfx/ranged.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(0.6),
            ..Default::default()
        }
    });
}

pub fn ensure_base_map(
    mut commands: Commands,
    map_query: Query<Entity, With<Map>>,
    mut map_grid: ResMut<MapGrid>,
    game_textures: Res<GameTextures>,
) {
    if !map_query.is_empty() {
        return;
    }

    commands
        .spawn((
            SpriteBundle {
                texture: game_textures.map.clone(),
                transform: Transform {
                    scale: Vec3::new(4.0, 4.0, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            Map,
            Resettable,
        ));

    map_grid.positions.insert((0, 0));
}


pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<NextState<GameState>>) {
    commands.spawn(Camera2dBundle::default());
    state.set(GameState::Menu);


    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        line: asset_server.load(LINE_SPRITE),
        map: asset_server.load(MAP_SPIRITE),
    };

    let enemy_count = EnemySpawnRate(2.0);

    let mouse_coords = MouseCoords {
        x: 0.,
        y: 0.,
    };

    // Create an entity dedicated to playing our background music
    commands.spawn(AudioBundle {
        source: asset_server.load("./beats/back.ogg"),
        settings: PlaybackSettings::LOOP,
    });
    commands.insert_resource(game_textures);
    commands.insert_resource(enemy_count);
    commands.insert_resource(mouse_coords);
    commands.insert_resource(Points::default());
}

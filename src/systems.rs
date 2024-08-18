use bevy::input::keyboard::Key;
use bevy::input::mouse::{self, MouseMotion};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_quickmenu::*;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;
use bevy::ui::{AlignItems, JustifyContent, Val, UiRect, Style};
use crate::components::{Direction, DirectionComponent, MovementSpeed, CollisionBox, Player, Tag, EnemySpawnTimer};
use crate::events::{CollisionEvent, Score};
use crate::{Ability, CooldownUi, Cooldowns, CurrentGameState, GameState, HealthText, Invulnerability, Lifetime, MousePosition, PauseMenu, PointMarker, Points, ScoreText};
use crate::Line;
use rand::Rng;
use std::f32::consts::PI;

// Systems Implementation

pub fn spawn_enemies_over_time(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>, // Query to get the player's Transform
    mut spawn_timer: ResMut<EnemySpawnTimer>,
) {
    // Update the timer
    spawn_timer.timer.tick(time.delta());

    // If the timer has finished and we haven't spawned all enemies
    if spawn_timer.timer.finished() && spawn_timer.enemies_spawned < spawn_timer.total_enemies {
        if let Ok(player_transform) = player_query.get_single() {
            let player_position = player_transform.translation;

            // Generate a random angle between 0 and 2π radians (full circle)
            let mut rng = rand::thread_rng();
            let angle = rng.gen_range(0.0..(2.0 * PI));

            // Calculate the x and y position based on the angle and radius
            let x = player_position.x + spawn_timer.spawn_radius * angle.cos();
            let y = player_position.y + spawn_timer.spawn_radius * angle.sin();

            // Spawn the enemy entity at the calculated position
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("../assets/pink_box.png"), // Enemy texture
                    transform: Transform::from_xyz(x, y, 0.0),           // Set position
                    ..Default::default()
                },
            ))
            .insert(CollisionBox::new(50.0, 50.0)) // Add collision box
            .insert(Tag { name: format!("Enemy{}", spawn_timer.enemies_spawned) }) // Tag with a unique name
            .insert(MovementSpeed(50.0)) // Set movement speed
            .insert(DirectionComponent { direction: Direction::None }); // Set initial direction

            // Increment the count of spawned enemies
            spawn_timer.enemies_spawned += 1;
        }
    }
}

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
    mut player_query: Query<(Entity, &mut Player, &CollisionBox, &Transform, Option<Mut< Invulnerability>>)>,    
    other_entities_query: Query<(Entity, &Transform, &CollisionBox), (Without<Player>, Without<Line>)>,
    mut collision_events: EventWriter<CollisionEvent>,
    points_query: Query<(Entity, &Transform), With<PointMarker>>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    mut points: ResMut<Points>,  
    mut despawned_entities: Local<HashSet<Entity>>,  // Track despawned entities
    line_query: Query<(Entity, &Transform, &CollisionBox), With<Line>>,
    mut exit: EventWriter<AppExit>, // Add the AppExit event writer
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
    for (attack_entity, attack_transform, attack_box) in line_query.iter() {
        let attack_min_x = attack_transform.translation.x - attack_box.width / 2.0;
        let attack_max_x = attack_transform.translation.x + attack_box.width / 2.0;
        let attack_min_y = attack_transform.translation.y - attack_box.height / 2.0;
        let attack_max_y = attack_transform.translation.y + attack_box.height / 2.0;

        for (enemy_entity, enemy_transform, enemy_box) in other_entities_query.iter() {
            let enemy_min_x = enemy_transform.translation.x - enemy_box.width / 2.0;
            let enemy_max_x = enemy_transform.translation.x + enemy_box.width / 2.0;
            let enemy_min_y = enemy_transform.translation.y - enemy_box.height / 2.0;
            let enemy_max_y = enemy_transform.translation.y + enemy_box.height / 2.0;

            if attack_max_x > enemy_min_x
                && attack_min_x < enemy_max_x
                && attack_max_y > enemy_min_y
                && attack_min_y < enemy_max_y
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

    
    for (entity, mut player, player_box, player_transform, mut invulnerability_option) in player_query.iter_mut() {
        let player_min_x = player_transform.translation.x - player_box.width / 2.0;
        let player_max_x = player_transform.translation.x + player_box.width / 2.0;
        let player_min_y = player_transform.translation.y - player_box.height / 2.0;
        let player_max_y = player_transform.translation.y + player_box.height / 2.0;

        if let Some(ref mut invulnerability) = invulnerability_option {
            if invulnerability.is_active() {
                continue; // Skip damage application if invulnerable
            }
        }

        for (enemy_entity, enemy_transform, enemy_box) in other_entities_query.iter() {
            let enemy_min_x = enemy_transform.translation.x - enemy_box.width / 2.0;
            let enemy_max_x = enemy_transform.translation.x + enemy_box.width / 2.0;
            let enemy_min_y = enemy_transform.translation.y - enemy_box.height / 2.0;
            let enemy_max_y = enemy_transform.translation.y + enemy_box.height / 2.0;

            if player_max_x > enemy_min_x
                && player_min_x < enemy_max_x
                && player_max_y > enemy_min_y
                && player_min_y < enemy_max_y
            {

            if player_max_x > enemy_min_x
                && player_min_x < enemy_max_x
                && player_max_y > enemy_min_y
                && player_min_y < enemy_max_y
            {
                // Handle collision, but only if player is not invulnerable
                player.take_damage(100, entity, &mut commands, invulnerability_option.as_deref_mut(), 0.5, &mut exit);
            }
        }
    }
}
}


pub fn handle_escape_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    mut curr_state: ResMut<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("gaming");
        if *curr_state.get() == GameState::Running {
            state.set(GameState::Paused);
        } else if *curr_state.get() == GameState::Paused {
            state.set(GameState::Running);
        }
    }
}

pub fn pathfind_towards_player(
    player_query: Query<&Transform, With<Player>>, // Get the player's transform
    mut enemy_query: Query<(&mut DirectionComponent, &Transform), Without<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut direction_component, enemy_transform) in enemy_query.iter_mut() {
            let direction_vector = (player_transform.translation - enemy_transform.translation).normalize();

            direction_component.direction = if direction_vector.x > 0.0 && direction_vector.y > 0.0 {
                Direction::UpRight
            } else if direction_vector.x < 0.0 && direction_vector.y > 0.0 {
                Direction::UpLeft
            } else if direction_vector.x > 0.0 && direction_vector.y < 0.0 {
                Direction::DownRight
            } else if direction_vector.x < 0.0 && direction_vector.y < 0.0 {
                Direction::DownLeft
            } else if direction_vector.x > 0.0 {
                Direction::Right
            } else if direction_vector.x < 0.0 {
                Direction::Left
            } else if direction_vector.y > 0.0 {
                Direction::Up
            } else if direction_vector.y < 0.0 {
                Direction::Down
            } else {
                Direction::None
            };
        }
    }
}

pub fn flicker_system(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut Invulnerability), With<Player>>,
) {
    for (mut sprite, mut invulnerability) in query.iter_mut() {
        // Update the timer for invulnerability
        invulnerability.timer.tick(time.delta());

        // If the player is invulnerable, adjust the alpha value to create a flicker effect
        if invulnerability.is_active() {
            // Flicker by adjusting alpha value between 0.2 and 1.0
            let flicker_phase = (invulnerability.timer.elapsed_secs() * 10.0).sin();
            let new_alpha = 0.5 * flicker_phase.abs();

            // Directly set the alpha using set_alpha
            sprite.color.set_alpha(new_alpha);
        } else {
            // Ensure the player is fully visible when not invulnerable
            sprite.color.set_alpha(1.0);
        }
    }
}

pub fn move_entities(
    time: Res<Time>,
    mut query: Query<(&DirectionComponent, &mut Transform, &MovementSpeed)>,
) {
    for (direction_component, mut transform, speed) in query.iter_mut() {
        let delta = speed.0 * time.delta_seconds();

        match direction_component.direction {
            Direction::None => {},
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

pub fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for (mut logo, mut transform) in &mut sprite_position {
        let delta = 150. * time.delta_seconds(); //movespeed

        //6dof movement handling
        if keyboard_input.pressed(KeyCode::KeyA) {
            if keyboard_input.pressed(KeyCode::KeyW) {
                *logo = Direction::UpLeft;
            }
            else if   keyboard_input.pressed(KeyCode::KeyS) {
                *logo = Direction::DownLeft;
            }
            else {
                *logo = Direction::Left;
            }
        }
        else if keyboard_input.pressed(KeyCode::KeyD) {
            if keyboard_input.pressed(KeyCode::KeyW) {
                *logo = Direction::UpRight;
            }
            else if   keyboard_input.pressed(KeyCode::KeyS) {
                *logo = Direction::DownRight;
            }
            else {
                *logo = Direction::Right;
            }
        }
        else if keyboard_input.pressed(KeyCode::KeyS) {
            *logo = Direction::Down;
        }
        else if keyboard_input.pressed(KeyCode::KeyW) {
            *logo = Direction::Up;
        }
        else {
            *logo = Direction::None;
        }

        //applying velocity to image transform
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

// System to update the MousePosition resource whenever the mouse moves
pub fn update_mouse_position(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_position: ResMut<MousePosition>,
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

// System to update the Player's position whenever the player's position changes
pub fn update_player_position(
    mut player_query: Query<(&Transform, &mut Player)>,
) {
    if let Ok((transform, mut player)) = player_query.get_single_mut() {
        player.update_position(transform);

       // println!("Player Position in World: ({}, {})", player.x, player.y);
    }
}
    
pub fn use_ability(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cooldown_query: Query<&mut Cooldowns>,
    mut player_query: Query<(Entity, &mut Transform, &mut Player)>,
    mouse_position: Res<MousePosition>,
    asset_server: Res<AssetServer>,
    mut points: ResMut<Points>,
) {
    if let Ok(mut cooldowns) = cooldown_query.get_single_mut() {
        if let Ok((player_entity, mut transform, mut player)) = player_query.get_single_mut() {
        let player_position = Vec2::new(transform.translation.x, transform.translation.y);

        // Line-drawing ability (mapped to the F key)
        if keyboard_input.just_pressed(KeyCode::KeyE) {
            if cooldowns.is_ready(Ability::Ranged) {
                ranged_attack(
                    commands,
                    player_position,
                    Vec2::new(mouse_position.x, mouse_position.y),
                    asset_server,
                );
                cooldowns.reset(Ability::Ranged);
            } else {
                println!("Line ability is on cooldown!");
            }
        }
        // Line-drawing ability (mapped to the F key)
        else if keyboard_input.just_pressed(KeyCode::KeyF) {
            if cooldowns.is_ready(Ability::Dash) {
                dash_attack(
                    commands,
                    player_position,
                    Vec2::new(mouse_position.x, mouse_position.y),
                    asset_server,
                    player_query,
                );
                cooldowns.reset(Ability::Dash);
            } else {
                println!("Dash is on cooldown!");
            }
        }
        // Arc-drawing ability (mapped to the E key)
        else if keyboard_input.just_pressed(KeyCode::KeyQ) {
            if cooldowns.is_ready(Ability::Attack) {  
                draw_arc_ability(
                    commands,
                    player_position,
                    Vec2::new(mouse_position.x, mouse_position.y),
                    asset_server,
                    points,
                );
                cooldowns.reset(Ability::Attack);
            } else {
                println!("Arc ability is on cooldown!");
            }
        }
        // Circle-drawing ability (mapped to the T key)
        else if keyboard_input.just_pressed(KeyCode::KeyT) {
            if cooldowns.is_ready(Ability::Aoe) {  
                draw_circle_ability(
                    commands,
                    player_position,
                    asset_server,
                    points,
                );
                cooldowns.reset(Ability::Aoe);
            } else {
                println!("Circle ability is on cooldown!");
            }
        }
        // Other abilities can be added here similarly...
    }
}
}


fn ranged_attack(
    mut commands: Commands,
    player_position: Vec2,
    mouse_position: Vec2,
    asset_server: Res<AssetServer>,
) {
    
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

    // Load the texture for the line
    let line_texture_handle = asset_server.load("red_line.png");

    // Spawn the line as a sprite with a bounding box
    commands.spawn(SpriteBundle {
        texture: line_texture_handle,
        transform: Transform {
            translation: Vec3::new(midpoint.x, midpoint.y, 0.0), // Center the line between the player and the endpoint
            rotation: Quat::from_rotation_z(angle),   // Rotate to face the endpoint
            scale: Vec3::new(line_length, 2.0, 1.0),       // Scale to the length, and set the thickness
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Line)
    .insert(CollisionBox::new(line_length, 50.0)) // Adjust the bounding box to cover the entire line
    .insert(Lifetime {
        timer: Timer::from_seconds(0.1, TimerMode::Once),
    });
}

fn dash_attack(
    mut commands: Commands,
    player_position: Vec2,
    mouse_position: Vec2,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &mut Transform, &mut Player)>,
) {
    if let Ok((player_entity, mut transform, mut player)) = player_query.get_single_mut() {
        let direction = mouse_position - player_position;
        let length = direction.length();

        // Calculate the midpoint between the player and the mouse
        let midpoint = player_position + direction / 2.0;

        // Correct rotation angle
        let angle = direction.y.atan2(direction.x);

        // Load a small texture for the line (e.g., 1x1 pixel)
        let line_texture_handle = asset_server.load("red_line.png");

        // Spawn the line as a sprite with a bounding box and a lifetime
        commands.spawn(SpriteBundle {
            texture: line_texture_handle,
            transform: Transform {
                translation: Vec3::new(midpoint.x, midpoint.y, 0.0), // Center the line between the player and the mouse
                rotation: Quat::from_rotation_z(angle),   // Rotate to face the mouse position
                scale: Vec3::new(length, 2.0, 1.0),       // Scale to the length, and set the thickness
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Line)
        .insert(CollisionBox::new(length, 50.0)) // Add a bounding box for collision detection
        .insert(Lifetime {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        });

        // Make the player invulnerable for 0.5 seconds
        commands.entity(player_entity).insert(Invulnerability {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        });

        player.move_to(mouse_position.x, mouse_position.y, &mut transform);
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


fn draw_arc_ability(
    mut commands: Commands,
    player_position: Vec2,
    mouse_position: Vec2,
    asset_server: Res<AssetServer>,
    mut points: ResMut<Points>,
) {
    let direction = (mouse_position - player_position).normalize();
    let start_angle = direction.y.atan2(direction.x);

    let max_radius = 250.0; // Max radius for the arc
    let theta = 0.0725; // Smaller theta for finer increments
    let arc_span = PI / 2.0; // 90 degrees in radians
    let radius_step = 10.0; // Distance between each concentric arc

    let arc_segments = (arc_span / theta) as i32; // Number of segments for 90 degrees

    points.0.clear();

    for radius in (radius_step as i32..=max_radius as i32).step_by(radius_step as usize) {
        for i in 0..=arc_segments {
            let angle = start_angle - (arc_span / 2.0) + i as f32 * theta;
            let arc_point = Vec2::new(
                player_position.x + radius as f32 * angle.cos(),
                player_position.y + radius as f32 * angle.sin(),
            );

            points.0.push(arc_point);

            // Draw a small circle or segment at the arc point
            commands.spawn(SpriteBundle {
                texture: asset_server.load("red_line.png"),
                transform: Transform {
                    translation: Vec3::new(arc_point.x, arc_point.y, 0.0),
                    scale: Vec3::new(5.0, 5.0, 1.0), // Adjust size as needed
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

fn draw_circle_ability(
    mut commands: Commands,
    player_position: Vec2,
    asset_server: Res<AssetServer>,
    mut points: ResMut<Points>,
) {
    let max_radius = 350.0; // Maximum radius for the circle
    let theta = 0.0725; // Smaller theta for finer increments
    let total_angle = 2.0 * PI; // Full circle (360 degrees)
    let radius_step = 10.0; // Distance between each concentric circle
    let arc_segments = (total_angle / theta) as i32; // Number of segments for the full circle

    points.0.clear();

    // Iterate over increasing radii to fill the circle
    for radius in (0..=max_radius as i32).step_by(radius_step as usize) {
        for i in 0..=arc_segments {
            let angle = i as f32 * theta;
            let circle_point = Vec2::new(
                player_position.x + radius as f32 * angle.cos(),
                player_position.y + radius as f32 * angle.sin(),
            );

            points.0.push(circle_point);

            // Draw a small circle or segment at the circle point
            commands.spawn(SpriteBundle {
                texture: asset_server.load("red_line.png"),
                transform: Transform {
                    translation: Vec3::new(circle_point.x, circle_point.y, 0.0),
                    scale: Vec3::new(5.0, 5.0, 1.0), // Adjust size as needed
                    ..Default::default()
                },
                ..Default::default()
            }).insert(PointMarker)
            .insert(Lifetime {
                timer: Timer::from_seconds(0.1, TimerMode::Once),
            });
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
    time: Res<Time>,
    mut cooldowns_query: Query<&mut Cooldowns>,
    mut text_query: Query<&mut Text, With<CooldownUi>>,
) {
    if let Ok(mut cooldowns) = cooldowns_query.get_single_mut() {
        // Update the UI text for each ability
        for (i, mut text) in text_query.iter_mut().enumerate() {
            let ability_text = match i {
                0 => format!("Attack: {:.1}s", cooldowns.get_cooldown(Ability::Attack).unwrap_or(0.0)),
                1 => format!("Ranged: {:.1}s", cooldowns.get_cooldown(Ability::Ranged).unwrap_or(0.0)),
                2 => format!("Dash: {:.1}s", cooldowns.get_cooldown(Ability::Dash).unwrap_or(0.0)),
                3 => format!("Aoe: {:.1}s", cooldowns.get_cooldown(Ability::Aoe).unwrap_or(0.0)),
                _ => "Unknown Ability".to_string(),
            };

            text.sections[0].value = ability_text;
        }
    }
}

pub fn update_ui_text(
    player_query: Query<&Player>,
    score: Res<Score>,
    mut text_query: Query<(&mut Text, Option<&HealthText>, Option<&ScoreText>)>,
) {
    if let Ok(player) = player_query.get_single() {
        for (mut text, health_text, score_text) in text_query.iter_mut() {
            if health_text.is_some() {
                text.sections[0].value = format!("Health: {}", player.health);
            } else if score_text.is_some() {
                text.sections[0].value = format!("Score: {}", score.get_enemies_killed());
            }
        }
    }
}

pub fn show_pause_menu(mut query: Query<&mut Visibility, With<PauseMenu>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

pub fn hide_pause_menu(mut query: Query<&mut Visibility, With<PauseMenu>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

pub fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(), // Semi-transparent background
            visibility: Visibility::Hidden, // Initially hidden
            ..Default::default()
        },
        PauseMenu,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text::from_section(
                "Game Paused\nPress Esc to Resume",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        });
    });
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<NextState<GameState>>) {
    commands.spawn(Camera2dBundle::default());
    state.set(GameState::Running);
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
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            ..Default::default()
        },
        ..Default::default()
    })
    .with_children(|parent| {
        // Health and Score container
        parent.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column, // Stack vertically
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(50.0), Val::Px(0.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            // Health Text
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Health: 500",
                    TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                ),
                ..Default::default()
            })
            .insert(HealthText);

            // Score Text
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Score: 0",
                    TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                ),
                style: Style {
                    //margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(100.0), Val::Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ScoreText);
        });

        // Ability boxes container at the bottom
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0), 
                height: Val::Px(60.0), // 60px high for the ability boxes
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0), // Position at the bottom of the screen
                justify_content: JustifyContent::SpaceAround, // Evenly space ability boxes
                align_items: AlignItems::Center, // Center the boxes vertically within the container
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            let abilities = [
                Ability::Attack,
                Ability::Ranged,
                Ability::Dash,
                Ability::Aoe,
            ];

            for ability in abilities.iter() {
                let ability_name = match ability {
                    Ability::Attack => "Attack",
                    Ability::Ranged => "Ranged",
                    Ability::Dash => "Dash",
                    Ability::Aoe => "Bladestorm",
                };

                parent.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(20.0),
                        height: Val::Px(50.0), // 50px height for each ability box
                        margin: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::srgba(0.9, 0.9, 0.9, 0.5).into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("{}: {:.1}s", ability_name, 0.0), // Ability name and placeholder cooldown
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 30.0,
                                color: Color::BLACK,
                            },
                        ),
                        ..Default::default()
                    })
                    .insert(CooldownUi);
                });
            }
        });
    });

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/blue_box.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            sprite: Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 1.0), // Ensure the sprite starts fully visible
                ..Default::default()
            },
            ..default()
        },
        Direction::None,
    ))
    .insert(CollisionBox::new(50.0, 50.0))
    .insert(Player::new(500))
    .insert(Cooldowns::new()); // Initialize cooldowns for abilities)
    //.insert(Invulnerability::new(0.3));

    //state.state = GameState::Running;    
    commands.insert_resource(EnemySpawnTimer::new(10, 750.)); 
}


use bevy::input::mouse::{self, MouseMotion};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashSet;
use bevy::window::PrimaryWindow;
use crate::components::{Direction, DirectionComponent, MovementSpeed, CollisionBox, Player, Tag, EnemySpawnTimer};
use crate::events::{CollisionEvent, Score};
use crate::{Ability, Cooldowns, Lifetime, MousePosition, PointMarker, Points};
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
    points_query: Query<(Entity, &Transform), With<PointMarker>>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    mut points: ResMut<Points>,  // Now accessed as mutable
    mut despawned_entities: Local<HashSet<Entity>>,  // Track despawned entities
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

                //despawned_entities.insert(enemy_entity);  // Track despawned entity

                // // Despawn all points and clear the resource
                // for (point_entity, _) in points_query.iter() {
                //     if despawned_entities.contains(&point_entity) {
                //         continue; // Skip if already despawned
                //     }
                //     commands.entity(point_entity).despawn();
                //     despawned_entities.insert(point_entity);  // Track despawned point
                // }
                // points.0.clear(); // Clear the stored points in the resource
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
    for (player_transform, player_collision_box, mut player) in player_query.iter_mut() {
        for (entity, other_transform, other_collision_box) in other_entities_query.iter() {
            if player_collision_box.intersects(player_transform, other_collision_box, other_transform) {
                collision_events.send(CollisionEvent(entity));
                player.take_damage(10);
            }
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

// System to draw a line from the player's position to the mouse position when 'Q' is pressed
// pub fn dash_attack(
//     mut commands: Commands,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mouse_position: Res<MousePosition>,
//     mut player_query: Query<(&mut Transform, &mut Player)>,
//     asset_server: Res<AssetServer>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyQ) {
//         if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
//             let player_position = Vec2::new(player.x, player.y);
//             let mouse_position = Vec2::new(mouse_position.x, mouse_position.y);

//             let direction = mouse_position - player_position;
//             let length = direction.length();

//             // Calculate the midpoint between the player and the mouse
//             let midpoint = player_position + direction / 2.0;

//             // Correct rotation angle
//             let angle = direction.y.atan2(direction.x);

//             // Load a small texture for the line (e.g., 1x1 pixel)
//             let line_texture_handle = asset_server.load("red_line.png");

//             // Spawn the line as a sprite with a bounding box
//             commands.spawn(SpriteBundle {
//                 texture: line_texture_handle,
//                 transform: Transform {
//                     translation: Vec3::new(midpoint.x, midpoint.y, 0.0), // Center the line between the player and the mouse
//                     rotation: Quat::from_rotation_z(angle),   // Rotate to face the mouse position
//                     scale: Vec3::new(length, 2.0, 1.0),       // Scale to the length, and set the thickness
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             })
//             .insert(Line)
//             .insert(CollisionBox::new(length, 20.0))
//             .insert(Lifetime {
//             timer: Timer::from_seconds(0.1, TimerMode::Once),
//             });

//             player.move_to(mouse_position.x, mouse_position.y, &mut transform);
//         }
//     }
// }

// System to draw a line from the player's position to the mouse position when 'Q' is pressed
// pub fn ranged_attack(
//     mut commands: Commands,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mouse_position: Res<MousePosition>,
//     mut player_query: Query<(&mut Transform, &mut Player)>,
//     asset_server: Res<AssetServer>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyF) {
//         if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
//             let player_position = Vec2::new(player.x, player.y);
//             let mouse_position = Vec2::new(mouse_position.x, mouse_position.y);

//             let direction = mouse_position - player_position;
//             let length = direction.length();

//             // Calculate the midpoint between the player and the mouse
//             let midpoint = player_position + direction / 2.0;

//             // Correct rotation angle
//             let angle = direction.y.atan2(direction.x);

//             // Load a small texture for the line (e.g., 1x1 pixel)
//             let line_texture_handle = asset_server.load("red_line.png");

//             // Spawn the line as a sprite with a bounding box
//             commands.spawn(SpriteBundle {
//                 texture: line_texture_handle,
//                 transform: Transform {
//                     translation: Vec3::new(midpoint.x, midpoint.y, 0.0), // Center the line between the player and the mouse
//                     rotation: Quat::from_rotation_z(angle),   // Rotate to face the mouse position
//                     scale: Vec3::new(length, 2.0, 1.0),       // Scale to the length, and set the thickness
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             })
//             .insert(Line)
//             .insert(CollisionBox::new(length, 20.0))
//             .insert(Lifetime {
//             timer: Timer::from_seconds(0.1, TimerMode::Once),
//             });

//         }
//     }
// }


    
    
pub fn use_ability(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cooldown_query: Query<&mut Cooldowns>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    mouse_position: Res<MousePosition>,
    asset_server: Res<AssetServer>,
    mut points: ResMut<Points>,
) {
    if let Ok(mut cooldowns) = cooldown_query.get_single_mut() {
        if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
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
            if cooldowns.is_ready(Ability::Attack) {  // 
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
            if cooldowns.is_ready(Ability::Aoe) {  // Ensure you add a DrawCircle variant to your Ability enum
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
    .insert(CollisionBox::new(1100.0, 50.0)) // Add a bounding box for collision detection
    .insert(Lifetime {
        timer: Timer::from_seconds(0.1, TimerMode::Once),
    });
}

fn dash_attack(
    mut commands: Commands,
    player_position: Vec2,
    mouse_position: Vec2,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
) {
    if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
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
        player.move_to(mouse_position.x, mouse_position.y, &mut transform);
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
// System to draw an arc from the player's position towards the mouse position when 'Q' is pressed
// pub // System to draw a filled arc from the player's position towards the mouse position when 'Q' is pressed
// fn draw_arc_on_e(
//     mut commands: Commands,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mouse_position: Res<MousePosition>,
//     player_query: Query<&Player>,
//     asset_server: Res<AssetServer>,
//     mut points: ResMut<Points>,
    
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyE) {
//         if let Ok(player) = player_query.get_single() {
//             let player_position = Vec2::new(player.x, player.y);
//             let mouse_position = Vec2::new(mouse_position.x, mouse_position.y);

//             let direction = (mouse_position - player_position).normalize();
//             let start_angle = direction.y.atan2(direction.x);

//             let max_radius = 250.0; // Max radius for the arc
//             let theta = 0.0725; // Smaller theta for finer increments
//             let arc_span = PI / 2.0; // 90 degrees in radians
//             let radius_step = 10.0; // Distance between each concentric arc

//             let arc_segments = (arc_span / theta) as i32; // Number of segments for 90 degrees

//             points.0.clear();

//             for radius in (radius_step as i32..=max_radius as i32).step_by(radius_step as usize) {
//                 for i in 0..=arc_segments {
//                     let angle = start_angle - (arc_span / 2.0) + i as f32 * theta;
//                     let arc_point = Vec2::new(
//                         player_position.x + radius as f32 * angle.cos(),
//                         player_position.y + radius as f32 * angle.sin(),
//                     );

//                     points.0.push(arc_point);

//                     // Draw a small circle or segment at the arc point
//                     commands.spawn(SpriteBundle {
//                         texture: asset_server.load("red_line.png"),
//                         transform: Transform {
//                             translation: Vec3::new(arc_point.x, arc_point.y, 0.0),
//                             scale: Vec3::new(5.0, 5.0, 1.0), // Adjust size as needed
//                             ..Default::default()
//                         },
//                         ..Default::default()

//                     }).insert(PointMarker)
//                     .insert(Lifetime {
//                         timer: Timer::from_seconds(0.1, TimerMode::Once),
//                         });  // Add this line;
//                 }
//             }
//         }
//     }
// }

// // System to draw a circle around the player's position with a radius of 350 pixels
// pub fn draw_circle_around_player(
//     mut commands: Commands,
//     player_query: Query<&Player>,
//     asset_server: Res<AssetServer>,
//     mut points: ResMut<Points>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyT) {
//         if let Ok(player) = player_query.get_single() {
//             let player_position = Vec2::new(player.x, player.y);
    
//             let max_radius = 350.0; // Maximum radius for the circle
//             let theta = 0.0725; // Smaller theta for finer increments
//             let total_angle = 2.0 * PI; // Full circle (360 degrees)
//             let radius_step = 10.0; // Distance between each concentric circle
//             let arc_segments = (total_angle / theta) as i32; // Number of segments for the full circle

//             points.0.clear();
    
//             // Iterate over increasing radii to fill the circle
//             for radius in (0..=max_radius as i32).step_by(radius_step as usize) {
//                 for i in 0..=arc_segments {
//                     let angle = i as f32 * theta;
//                     let circle_point = Vec2::new(
//                         player_position.x + radius as f32 * angle.cos(),
//                         player_position.y + radius as f32 * angle.sin(),
//                     );

//                     points.0.push(circle_point);
    
//                     // Draw a small circle or segment at the circle point
//                     commands.spawn(SpriteBundle {
//                         texture: asset_server.load("red_line.png"),
//                         transform: Transform {
//                             translation: Vec3::new(circle_point.x, circle_point.y, 0.0),
//                             scale: Vec3::new(5.0, 5.0, 1.0), // Adjust size as needed
//                             ..Default::default()
//                         },
//                         ..Default::default()
//                     }).insert(PointMarker)
//                     .insert(Lifetime {
//                         timer: Timer::from_seconds(0.1, TimerMode::Once),
//                         }); 
//                 }
//             }
//         }
//     }
//     }

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


pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(
        TextBundle::from_section(
            "WASD to Move around, E to Melee, Q to Dash, R to kill",
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

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/blue_box.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::None,
    ))
    .insert(CollisionBox::new(50.0, 50.0))
    .insert(Player::new(500))
    .insert(Cooldowns::new());  // Initialize cooldowns for abilities)

    commands.insert_resource(EnemySpawnTimer::new(10, 750.)); 
}

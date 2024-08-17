use bevy::prelude::*;
use crate::components::{Direction, DirectionComponent, MovementSpeed, CollisionBox, Player, Tag, EnemySpawnTimer};
use crate::events::{CollisionEvent, Score};
use rand::Rng;

// Systems Implementation

pub fn spawn_enemies_over_time(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
) {
    // Update the timer
    spawn_timer.timer.tick(time.delta());

    // If the timer has finished and we haven't spawned all enemies
    if spawn_timer.timer.finished() && spawn_timer.enemies_spawned < spawn_timer.total_enemies {
        // Generate random x and y positions
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-500.0..500.0);
        let y = rng.gen_range(-500.0..500.0);

        // Spawn the enemy entity
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("../assets/pink_box.png"), // Enemy texture
                transform: Transform::from_xyz(x, y, 0.0), // Set random position
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

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/blue_box.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::None,
    ))
    .insert(CollisionBox::new(50.0, 50.0))
    .insert(Player::new(500));

    commands.insert_resource(EnemySpawnTimer::new(10)); 
}

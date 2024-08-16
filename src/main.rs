//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;

#[derive(Component)]
#[derive(PartialEq)]
//movement directions
enum Direction {
    Up,
    UpLeft,
    Left,
    Right,
    UpRight,
    Down,
    DownLeft,
    DownRight,
    None
}

//player health
#[derive(Component)]
struct Player {
    health: i32,
}

impl Player {
    fn new(health: i32) -> Self {
        Player { health }
    }

    fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        println!("Player took {} damage, remaining health: {}", amount, self.health);
    }
}

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct MovementSpeed(f32);

#[derive(Component)]
struct DirectionComponent {
    direction: Direction,
}

//entity identification via str
#[derive(Component)]
struct Tag {
    name: String,
}

//struct for tracking score via kills
#[derive(Resource)] // Add this line
struct Score {
    enemies_killed: u32,
}

impl Score {
    fn new() -> Self {
        Score {
            enemies_killed: 0,
        }
    }

    fn increment(&mut self) {
        self.enemies_killed += 1;
    }

    fn get_enemies_killed(&self) -> u32 {
        self.enemies_killed
    }
}

//collision box logic
#[derive(Component)]
struct CollisionBox {
    width: f32,
    height: f32,
}
#[derive(Event)]
struct CollisionEvent(Entity); // Event carrying the entity to delete


//AABB
impl CollisionBox {
    fn new(width: f32, height: f32) -> Self {
        CollisionBox { width, height }
    }

    fn intersects(&self, transform: &Transform, other: &CollisionBox, other_transform: &Transform) -> bool {
        let self_min_x = transform.translation.x - self.width / 2.0;
        let self_max_x = transform.translation.x + self.width / 2.0;
        let self_min_y = transform.translation.y - self.height / 2.0;
        let self_max_y = transform.translation.y + self.height / 2.0;

        let other_min_x = other_transform.translation.x - other.width / 2.0;
        let other_max_x = other_transform.translation.x + other.width / 2.0;
        let other_min_y = other_transform.translation.y - other.height / 2.0;
        let other_max_y = other_transform.translation.y + other.height / 2.0;

        self_min_x < other_max_x &&
        self_max_x > other_min_x &&
        self_min_y < other_max_y &&
        self_max_y > other_min_y
    }
}

//can do some even on taking damage like invulnerability time or something
#[derive(Event)]
struct Damaged;

fn handle_collisions(
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

fn main() {
    App::new()
        .insert_resource(Score::new()) //add ability to modify score
        .add_plugins(DefaultPlugins)// pulls in default plugin list, ECS, 2d rendering etc
        .add_systems(Startup, setup)// make the initialize() using the setup function
        .add_systems(FixedUpdate, (sprite_movement, pathfind_towards_player, move_entities, display_score, check_collisions, handle_collisions)) // make the game loop run once a frame
        //FixedUpdate + chain means it runs in succession left to right of stuff in tuple
        .add_event::<CollisionEvent>()
        .run();
}

// System to simulate enemy kills
fn enemy_killed(score: &mut ResMut<Score>) {
    score.increment();
    println!("Score: {}", score.get_enemies_killed())
    //kill with EntityCommands::despawn || or Entities.free(entity)
}



// System to display the current score commented out because spam uncomment out _in front of _score to fix
fn display_score(_score: Res<Score>) {
    //println!("Enemies killed: {}", score.get_enemies_killed());
}

//check if player runs into something and takes away health if they are
fn check_collisions(
    mut player_query: Query<(&Transform, &CollisionBox, &mut Player)>,
    other_entities_query: Query<(Entity, &Transform, &CollisionBox), Without<Player>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    for (player_transform, player_collision_box, mut player) in player_query.iter_mut() {
        for (entity, other_transform, other_collision_box) in other_entities_query.iter() {
            if player_collision_box.intersects(player_transform, other_collision_box, other_transform) {
                collision_events.send(CollisionEvent(entity)); //check for e push and kill
                player.take_damage(10); // Player takes 10 damage on collision
            }
        }
    }
}



fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    //instructions in top left
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

    //let mut observer = Observer::new(take_damage);
    //main player sprite
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/blue_box.png"), //image
            transform: Transform::from_xyz(100., 0., 0.), //start pos
            ..default()
        },
        Direction::None, //initial velocity
    )).insert(CollisionBox::new(50.0, 50.0)) //attach collision box to entity
    .insert(Player::new(500));//attach health to entity

    // commands
    //     // Observers can watch for events targeting a specific entity.
    //     // This will create a new observer that runs whenever the Explode event
    //     // is triggered for this spawned entity.
    //     .observe(take_damage);

    //observer.watch_entity(player);


    //"enemy" sprite
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/pink_box.png"),
            transform: Transform::from_xyz(250., 250., 0.),
            ..default()
        },
        //chase_player()
    )).insert(CollisionBox::new(50.0, 50.0))
    .insert(Tag {
        name: "Enemy1".to_string(),
    })
    .insert(Position(Vec2::new(300.0, 300.0))) // Spawned at some position
    .insert(MovementSpeed(50.0))
    .insert(DirectionComponent { direction: Direction::None });;
    
    //commands.spawn(observer);
}

fn pathfind_towards_player(
    player_query: Query<&Transform, With<Player>>, // Get the player's transform
    mut enemy_query: Query<(&mut DirectionComponent, &Transform), Without<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut direction_component, enemy_transform) in enemy_query.iter_mut() {
            let direction_vector = (player_transform.translation - enemy_transform.translation).normalize();

            // Determine the direction based on the relative position to the player
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

fn move_entities(
    time: Res<Time>,
    mut query: Query<(&DirectionComponent, &mut Transform, &MovementSpeed)>,
) {
    for (direction_component, mut transform, speed) in query.iter_mut() {
        let delta = speed.0 * time.delta_seconds(); // Calculate movement delta based on speed and time

        // Apply velocity to the transform based on the direction
        match direction_component.direction {
            Direction::None => {}, // No movement if the direction is None
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

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>, keyboard_input: Res<ButtonInput<KeyCode>>) {
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

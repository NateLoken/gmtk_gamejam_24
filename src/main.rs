//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;

#[derive(Component)]
#[derive(PartialEq)]
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

#[derive(Resource)]
struct EntityIdTracker {
    current_id: u32,
}

impl EntityIdTracker {
    fn new() -> Self {
        EntityIdTracker { current_id: 0 }
    }

    fn get_next_id(&mut self) -> u32 {
        self.current_id += 1;
        self.current_id
    }
}

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

#[derive(Component)]
struct CollisionBox {
    width: f32,
    height: f32,
}

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

#[derive(Event)]
struct Damaged;

fn main() {
    App::new()
        .insert_resource(Score::new())
        .insert_resource(EntityIdTracker::new())
        .add_plugins(DefaultPlugins)// pulls in default plugin list, ECS, 2d rendering etc
        .add_systems(Startup, setup)// make the initialize() using the setup function
        .add_systems(FixedUpdate, (sprite_movement, enemy_killed, display_score, observe_transform_changes).chain()) // make the game loop using sprite_movement() function
        .run();
}

// System to simulate enemy kills
fn enemy_killed(mut score: ResMut<Score>) {
    score.increment();
}

// System to display the current score
fn display_score(score: Res<Score>) {
    //println!("Enemies killed: {}", score.get_enemies_killed());
}


//fn entity_pathfind_to_entity(mut )

fn take_damage(trigger: Trigger<Damaged>, query: Query<&CollisionBox>, mut commands: Commands) {
    return
}

fn observe_transform_changes(
    mut query: Query<(&Transform, &CollisionBox, Entity), Changed<Transform>>,
    all_entities: Query<(Entity, &Transform, &CollisionBox)>,
) {
    for (transform_a, collision_box_a, entity_a) in query.iter() {
        for (entity_b, transform_b, collision_box_b) in all_entities.iter() {
            if entity_a == entity_b {
                continue;
            }

            if collision_box_a.intersects(transform_a, collision_box_b, transform_b) {
                println!(
                    "Entity {:?} is colliding with Entity {:?}",
                    entity_a, entity_b
                );
            }
        }
    }
}



fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut id_tracker: ResMut<EntityIdTracker>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(
        TextBundle::from_section(
            "ArrowKeys to Move around",
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

    let mut observer = Observer::new(take_damage);
    let player = commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/blue_box.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::None,
    )).insert(CollisionBox::new(50.0, 50.0))
    .id();

    commands
        // Observers can watch for events targeting a specific entity.
        // This will create a new observer that runs whenever the Explode event
        // is triggered for this spawned entity.
        .observe(take_damage);

    observer.watch_entity(player);

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/pink_box.png"),
            transform: Transform::from_xyz(250., 250., 0.),
            ..default()
        },
        //chase_player()
    )).insert(CollisionBox::new(50.0, 50.0));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for (mut logo, mut transform) in &mut sprite_position {
        let delta = 150. * time.delta_seconds();

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                *logo = Direction::UpLeft;
            }
            else if   keyboard_input.pressed(KeyCode::ArrowDown) {
                *logo = Direction::DownLeft;
            }
            else {
                *logo = Direction::Left;
            }
        }
        else if keyboard_input.pressed(KeyCode::ArrowRight) {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                *logo = Direction::UpRight;
            }
            else if   keyboard_input.pressed(KeyCode::ArrowDown) {
                *logo = Direction::DownRight;
            }
            else {
                *logo = Direction::Right;
            }
        }
        else if keyboard_input.pressed(KeyCode::ArrowDown) {
            *logo = Direction::Down;
        }
        else if keyboard_input.pressed(KeyCode::ArrowUp) {
            *logo = Direction::Up;
        }
        else {
            *logo = Direction::None;
        }

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



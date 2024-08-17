use bevy::prelude::*;

#[derive(Component, PartialEq)]
pub enum Direction {
    Up,
    UpLeft,
    Left,
    Right,
    UpRight,
    Down,
    DownLeft,
    DownRight,
    None,
}
#[derive(Component)]
pub struct CooldownUi;

#[derive(Component)]
pub struct Line;

#[derive(Component)]
pub struct Player {
    pub health: i32,
    pub x: f32,
    pub y: f32,
}

impl Player {
    pub fn new(health: i32) -> Self {
        Player { health, x: 0., y: 0. }
    }

    pub fn take_damage(
        &mut self,
        amount: i32,
        entity: Entity,
        commands: &mut Commands,
        invulnerability_option: Option<&mut Invulnerability>,
        invulnerability_duration: f32,
    ) {
        if let Some(invulnerability) = invulnerability_option {
            if invulnerability.is_active() {
                println!("Player is invulnerable, no damage taken.");
                return;
            } else {
                invulnerability.reset(); // Reset the timer if it's not active
                println!("Invulnerability reset.");
            }
        } else {
            // If no invulnerability component, add it with the desired duration
            commands.entity(entity).insert(Invulnerability::new(invulnerability_duration));
            println!("Invulnerability added with duration: {} seconds.", invulnerability_duration);
        }

        // Apply damage to the player
        self.health -= amount;
        println!("Player took {} damage, remaining health: {}", amount, self.health);
    }

    pub fn update_position(&mut self, transform: &Transform) {
        self.x = transform.translation.x;
        self.y = transform.translation.y;
    }

    pub fn move_to(&mut self, x: f32, y: f32, transform: &mut Transform) {
        self.x = x;
        self.y = y;
        transform.translation.x = x;
        transform.translation.y = y;
    }
}

#[derive(Component)]
pub struct PointMarker;

#[derive(Component)]
pub struct Lifetime {
    pub(crate) timer: Timer,
}

// Add the EnemySpawnTimer struct
#[derive(Component)]
#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: Timer,
    pub enemies_spawned: usize,
    pub total_enemies: usize,
    pub spawn_radius: f32,
}

impl EnemySpawnTimer {
    pub fn new(total_enemies: usize, spawn_radius: f32) -> Self {
        EnemySpawnTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            enemies_spawned: 0,
            total_enemies,
            spawn_radius,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Dash,
    Attack,
    Ranged,
    Aoe,
}

use std::collections::HashMap;

#[derive(Component)]
pub struct Cooldowns {
    pub cooldowns: HashMap<Ability, Timer>,
}

impl Cooldowns {
    pub fn new() -> Self {
        let mut cooldowns = HashMap::new();
        cooldowns.insert(Ability::Dash, Timer::from_seconds(5.0, TimerMode::Once)); // 5 second cooldown
        cooldowns.insert(Ability::Ranged, Timer::from_seconds(3.0, TimerMode::Once));
        cooldowns.insert(Ability::Attack, Timer::from_seconds(1.0, TimerMode::Once));    // 3 second cooldown
        cooldowns.insert(Ability::Aoe, Timer::from_seconds(10.0, TimerMode::Once)); // 10 second cooldown
        Self { cooldowns }
    }

    pub fn is_ready(&self, ability: Ability) -> bool {
        if let Some(timer) = self.cooldowns.get(&ability) {
            timer.finished()
        } else {
            false
        }
    }

    pub fn reset(&mut self, ability: Ability) {
        if let Some(timer) = self.cooldowns.get_mut(&ability) {
            timer.reset();
        }
    }

    pub fn get_cooldown(&mut self, ability: Ability) -> Option<f32> {
        if let Some(timer) = self.cooldowns.get_mut(&ability) {
            Some(timer.remaining_secs())
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct DirectionComponent {
    pub direction: Direction,
}

#[derive(Default, Resource)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Resource)]
pub struct Points(pub Vec<Vec2>);


#[derive(Component)]
pub struct Tag {
    pub name: String,
}

#[derive(Component)]
pub struct Invulnerability {
    pub timer: Timer,
}

impl Invulnerability {
    pub fn new(duration: f32) -> Self {
        Invulnerability {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn is_active(&self) -> bool {
        !self.timer.finished()
    }

    pub fn reset(&mut self) {
        self.timer.reset();
    }
}


#[derive(Component)]
pub struct CollisionBox {
    pub width: f32,
    pub height: f32,
}

impl CollisionBox {
    pub fn new(width: f32, height: f32) -> Self {
        CollisionBox { width, height }
    }

    pub fn intersects(&self, transform: &Transform, other: &CollisionBox, other_transform: &Transform) -> bool {
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

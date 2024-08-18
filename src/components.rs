//use bevy::prelude::{Component, Transform};
use bevy::prelude::*;
use std::collections::HashMap;

// Common Components
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
#[derive(Component)]
pub struct CooldownUi;

// Player Components
#[derive(Component)]
pub struct Line;

#[derive(Component)]
pub struct Player {
    pub health: i32,
    pub x: f64,
    pub y: f64,
}

impl Player {
    pub fn new(health: i32) -> Self {
        Player { health, x: 0., y: 0. }
    }

    pub fn heal(
        &mut self,
        amount: i32,
    ){
        if self.health < 500 {
            self.health +=amount;
        }
    }

    pub fn take_damage(
        &mut self,
        amount: i32,
        entity: Entity,
        commands: &mut Commands,
        invulnerability_option: Option<&mut Invulnerability>,
        invulnerability_duration: f32,
        exit: &mut EventWriter<AppExit>, 
    ) {
        if let Some(invulnerability) = invulnerability_option {
            if invulnerability.is_active() {
                //println!("Player is invulnerable, no damage taken.");
                return;
            } else {
                invulnerability.reset(); // Reset the timer if it's not active
                println!("Invulnerability reset.");
            }
        } else {
            // If no invulnerability component, add it with the desired duration
            commands.entity(entity).insert(Invulnerability::new(invulnerability_duration));
           // println!("Invulnerability added with duration: {} seconds.", invulnerability_duration);
        }

        // Apply damage to the player
        self.health -= amount;
        println!("Player took {} damage, remaining health: {}", amount, self.health);

        if self.health <= 0 {
            println!("Player has died. Exiting the game.");
            exit.send(AppExit::Success);
        }
    }
}

#[derive(Component)]
pub struct PointMarker;

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

// Enemy components
#[derive(Component)]
pub struct Enemy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Dash,
    Attack,
    Ranged,
    Aoe,
}

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

#[derive(Resource)]
pub struct Score {
    pub enemies_killed: u32,
}

impl Score {
    pub fn new() -> Self {
        Score {
            enemies_killed: 0,
        }
    }

    pub fn increment(&mut self) {
        self.enemies_killed += 1;
    }

    pub fn get_enemies_killed(&self) -> u32 {
        self.enemies_killed
    }
}
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Running,
    Menu,
    Paused,
}

#[derive(Resource)]
pub struct CurrentGameState {
    pub(crate) state: GameState,
}
#[derive(Component)]
pub struct PauseMenu;



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
pub struct HealthText;

#[derive(Component)]
pub struct ScoreText;


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

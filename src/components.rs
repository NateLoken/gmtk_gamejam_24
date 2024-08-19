//use bevy::prelude::{Component, Transform};
use bevy::{prelude::*, utils::{hashbrown::HashSet}};
use std::{collections::{HashMap}, fmt};

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
    pub x: f32,
    pub y: f32,
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
            invulnerability: Option<&mut Invulnerability>,
        ) {
            if let Some(mut invuln) = invulnerability {
                if invuln.is_active() {
                    println!("Player is invulnerable, no damage taken.");
                    return;
                } else {
                    // Reset the invulnerability timer
                    invuln.reset();
                }
            }
    
            // Apply the damage
            self.health -= amount;
            println!("Player took {} damage, remaining health: {}", amount, self.health);
    
            // Check for game over condition
            if self.health <= 0 {
                println!("Player is dead! Game Over.");
                // Handle game over logic here, such as triggering an event to exit the game
                // For example, you can trigger the game exit here:
                // commands.spawn().insert(GameOverEvent);
                // Or if you want to exit immediately:
                std::process::exit(0);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum Ability {
    Dash,
    Attack,
    Ranged,
    Aoe,
}
impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ability::Attack => write!(f, "Attack"),
            Ability::Ranged => write!(f, "Ranged"),
            Ability::Dash => write!(f, "Dash"),
            Ability::Aoe => write!(f, "Bladestorm"),
        }
    }
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

    pub fn get_cooldown(&self, ability: Ability) -> Option<f32> {
        if let Some(timer) = self.cooldowns.get(&ability) {
            let remaining_time = timer.duration().as_secs_f32() - timer.elapsed_secs();
            Some(remaining_time.max(0.0)) // Ensure it never goes negative
        } else {
            None
        }
    }
    
}

#[derive(Component)]
pub struct Map;

#[derive(Default, Resource)]
pub struct MapGrid {
    pub positions: HashSet<(i32, i32)>, // A set to track the positions of maps on the grid
}

#[derive(Component)]
pub struct Bigfoot {
    pub x: f32,
    pub y: f32,
    pub state: BigfootState,
    pub timer: Timer,
    pub health: i32,
}

impl Bigfoot {
    pub fn new(x: f32, y: f32) -> Self {
        Bigfoot {
            timer: Timer::from_seconds(2.5, TimerMode::Once),
            state: BigfootState::Invulnerable,
            x,
            y,
            health: 5, // Initial health value
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health < 0 {
            self.health = 0;
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }
}

#[derive(PartialEq)]
pub enum BigfootState {
    Invulnerable,
    Solid,
    Cleanup,
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

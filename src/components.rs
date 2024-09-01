use bevy::{asset::Handle, ecs::entity::Entity, prelude::{Component, Resource, Timer, TimerMode, Vec2}, render::texture::Image, state::state::States, utils::HashSet};
use std::{collections::HashMap, fmt, time::Duration};
use crate::death_sound;

// Common Components
#[derive(Component)]
pub struct Collider{
    pub size: Vec2,
    pub collisions: Vec<Entity>,
}

impl Collider {
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            collisions: vec![],
        }
    }

}

#[derive(Component)]
pub struct Health {
    pub hp: i32
}

impl Health {
    pub fn take_damage(&mut self, amount: i32) {
        self.hp -= amount;
        println!("Damage Taken: {}", amount);
    }
}

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
pub struct Player;

#[derive(Resource)]
pub struct SpawnTimer {
    pub timer: Timer,
    pub interval_decrease: f32,
}

impl SpawnTimer {
    pub fn new(initial_duration: Duration, interval_decrease: f32) -> Self {
        Self {
            timer: Timer::new(initial_duration, TimerMode::Repeating),
            interval_decrease,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        // Decrease the interval by the specified amount (2 milliseconds per second)
        let decrease = self.interval_decrease * delta.as_secs_f32();
        let new_duration = (self.timer.duration().as_secs_f32() - decrease).max(0.001); // Ensure the duration doesn't go below 1ms
        self.timer.set_duration(Duration::from_secs_f32(new_duration));
        self.timer.tick(delta);
    }
}

#[derive(Component)]
pub struct wallpaper;

//#[derive(Component)]
//pub struct Player {
//    pub health: i32,
//    pub x: f32,
//    pub y: f32,
//}

//impl Player {
//    pub fn new(health: i32) -> Self {
//        Player { health, x: 0., y: 0. }
//    }
//
//    pub fn heal(
//        &mut self,
//        amount: i32,
//    ){
//        if self.health < 500 {
//            self.health +=amount;
//        }
//    }
//
//    pub fn take_damage(
//        &mut self,
//        amount: i32,
//        entity: Entity,
//        commands: &mut Commands,
//        invulnerability_option: Option<&mut Invulnerability>,
//        state: &mut ResMut<NextState<GameState>>,
//        asset_server: & Res<AssetServer>,
//    ) {
//        if let Some(invulnerability) = invulnerability_option {
//            if invulnerability.is_active() {
//                //println!("Player is invulnerable, no damage taken.");
//                return;
//            } 
//        } else {
//            // If no invulnerability component, add it with the desired duration
//            commands.entity(entity).insert(Invulnerability::new(1.0));
//            // println!("Invulnerability added with duration: {} seconds.", invulnerability_duration);
//        }
//
//        // Apply damage to the player
//        self.health -= amount;
//        println!("Player took {} damage, remaining health: {}", amount, self.health);
//        >>>>>>> main
//
//            if self.health <= 0 {
//                death_sound(asset_server, commands);
//                state.set(GameState::GameOver);
//            }
//        commands.entity(entity).insert(Invulnerability::new(1.0));
//    }
//}

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

    pub fn reset_all(&mut self) {
        for timer in self.cooldowns.values_mut() {
            timer.reset();
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
    pub airTexture: Handle<Image>,
    pub groundTexture: Handle<Image>,
}

impl Bigfoot {
    pub fn new(x: f32, y: f32) -> Self {
        Bigfoot {
            timer: Timer::from_seconds(2.5, TimerMode::Once),
            state: BigfootState::Invulnerable,
            x,
            y,
            health: 5,
            airTexture: todo!(),
            groundTexture: todo!(), // Initial health value
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

#[derive(Component)]
pub struct GameOverUI;


#[derive(Component)]
pub struct Resettable;

#[derive(Component)]
pub struct GameUI;


#[derive(Resource)]
pub struct Score {
    pub enemies_killed: u32,
}

#[derive(Resource)]
pub struct GameTimer(pub f32);

#[derive(Component)]
pub struct GameTimerText;


impl Score {
    pub fn new() -> Self {
        Score {
            enemies_killed: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enemies_killed = 0;
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
    Reset,
    GameOver,
    Won
}

#[derive(Resource)]
pub struct CurrentGameState {
    pub(crate) state: GameState,
}
#[derive(Component)]
pub struct PauseMenu;

#[derive(Component, PartialEq)]
pub struct StartButton;

#[derive(Component, PartialEq)]
pub struct RestartButton;

#[derive(Component)]
pub struct QuitButton;


#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

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
pub struct Invulnerability {
    pub timer: Timer,
}

use bevy::prelude::{Component, Timer, TimerMode, Vec2, Resource};
use std::collections::HashMap;

// Common Components
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Collider;

// Player Components
#[derive(Component)]
pub struct Line;

#[derive(Component)]
pub struct Player {
    pub health: i32,
}

impl Player {
    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        println!("Player took {} damage, remaining health: {}", amount, self.health);
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

#[derive(Component)]
pub struct Boss;

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
}


#[derive(Default, Resource)]
pub struct Points(pub Vec<Vec2>);


#[derive(Component)]
pub struct Invulnerability {
    pub timer: Timer,
}



#[derive(Component)]
pub struct CollisionBox {
    pub width: f32,
    pub height: f32,
}

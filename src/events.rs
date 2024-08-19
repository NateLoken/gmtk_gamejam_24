use bevy::prelude::*;

#[derive(Event)]
pub enum CollisionEvent{
    Collision,
    Damage,
} // Event carrying the entity to delete

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

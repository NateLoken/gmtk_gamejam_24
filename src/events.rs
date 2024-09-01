use bevy::prelude::*;

#[derive(Event)]
pub enum CollisionEvent{
    Collision,
    Damage(Entity),
} // Event carrying the entity to delete


use bevy::prelude::*;

#[derive(Event)]
pub struct CollisionEvent(pub Entity); // Event carrying the entity to delete


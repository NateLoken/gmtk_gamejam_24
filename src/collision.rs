use std::collections::HashMap;
use bevy::prelude::*;
use crate::{components::{Collider, Enemy, Health, Line, Player, PointMarker, Velocity}, CollisionEvent, BASE_SPEED, TIME_STEP};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
         app.add_systems(Update, (detect_collisions, handle_collisions,));
    }    
}

fn detect_collisions(
    mut query: Query<(Entity, &Transform, &mut Collider, Option<&Player>, Option<&Enemy>, Option<&Line>, Option<&PointMarker>)>,
    mut events: EventWriter<CollisionEvent>,
) {
    let mut collisions: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (entity_a, transform_a, collider_a, player_a, enemy_a, line_a, point_marker_a) in query.iter() {
        let rect_a = Rect::from_center_size(transform_a.translation.truncate(), collider_a.size);

        for (entity_b, transform_b, collider_b, player_b, enemy_b, line_b, point_marker_b) in query.iter() {
            let rect_b = Rect::from_center_size(transform_b.translation.truncate(), collider_b.size);

            if entity_b == entity_a {
                continue;
            }

            if !rect_a.intersect(rect_b).is_empty() {
                if player_a.is_some() && enemy_b.is_some() {
                    events.send(CollisionEvent::Collision);

                    collisions.entry(entity_a).or_default().push(entity_b);

                } else if line_a.is_some() && enemy_b.is_some() {
                    events.send(CollisionEvent::Damage(entity_a));

                    collisions.entry(entity_a).or_default().push(entity_b);

                } else if point_marker_a.is_some() && enemy_b.is_some() {
                    events.send(CollisionEvent::Damage(entity_a));

                    collisions.entry(entity_a).or_default().push(entity_b);
                }
            }
        }

    }

    for(entity, _, mut collider, _, _, _, _) in query.iter_mut() {
        collider.collisions = collisions.remove(&entity).unwrap_or_default();
    }
}

fn handle_collisions(
    mut collision_reader: EventReader<CollisionEvent>,
    entity_query: Query<&Collider, Without<Player>>,
    mut player_query: Query<(&mut Collider, &mut Transform, &mut Health), With<Player>>,
    transform_query: Query<&Transform, Without<Player>>,
    mut health: Query<&mut Health, Without<Player>>,
) {
    for event in collision_reader.read() {
        match event {
            CollisionEvent::Collision => {
                if let Ok((mut player_collider, mut player_transform, mut player_health)) = player_query.get_single_mut() {
                    player_health.take_damage(10);
                    let mut direction_vector = Vec3::ZERO;

                    for collision in player_collider.collisions.iter() {
                        let enemy_transform = transform_query.get(*collision).expect("Collided with entity without collider");

                        let direction = (player_transform.translation - enemy_transform.translation).normalize();

                        direction_vector += direction;
                    }

                    player_collider.collisions.clear();

                    player_transform.translation += direction_vector * TIME_STEP * BASE_SPEED;

                }
            }
            CollisionEvent::Damage(entity) => {
                if let Ok(entity_collider) = entity_query.get(*entity) {
                    for collisions in entity_collider.collisions.iter() {
                        if let Ok(mut other_entity_health) = health.get_mut(*collisions) {
                            other_entity_health.take_damage(1);
                        }
                    }
                }
               
            }
        }    
    }
}

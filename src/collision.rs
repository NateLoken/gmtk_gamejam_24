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
                    events.send(CollisionEvent::Damage);

                    collisions.entry(entity_a).or_default().push(entity_b);

                } else if enemy_a.is_some() && enemy_b.is_some(){
                    events.send(CollisionEvent::Collision);

                   collisions.entry(entity_a).or_default().push(entity_b);

                } else if line_a.is_some() && enemy_b.is_some() {
                    events.send(CollisionEvent::Damage);

                    collisions.entry(entity_a).or_default().push(entity_b);

                } else if point_marker_a.is_some() && enemy_b.is_some() {
                    events.send(CollisionEvent::Damage);

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
    player: Query<Entity, With<Player>>,
    transforms: Query<&Transform>,
    health: Query<&Health>,
) {
    for event in collision_reader.read() {
        match event {
            CollisionEvent::Collision => {
               if let Ok((mut player_health, mut player_collider, mut player_transform)) = player_query.get_single_mut() {
                   player_health.take_damage(10);

                   let mut direction_vector = Vec3::ZERO;
                   for collision in  player_collider.collisions.iter() {
                       let enemy_transform = transforms.get(*collision).expect("Collided with entity without collider");

                       let direction = (player_transform.translation - enemy_transform.translation).normalize();

                       direction_vector += direction;
                   }

                   player_collider.collisions.clear();
                    
                   if direction_vector.x < 0. {
                       if direction_vector.y < 0. {
                           player_transform.translation.x -= direction_vector.x * TIME_STEP * BASE_SPEED;
                           player_transform.translation.y -= direction_vector.y * TIME_STEP * BASE_SPEED;
                       } else {
                           player_transform.translation.x -= direction_vector.x * TIME_STEP * BASE_SPEED;
                           player_transform.translation.y += direction_vector.y * TIME_STEP * BASE_SPEED;
                       }
                   } else {
                       player_transform.translation.x += direction_vector.x * TIME_STEP * BASE_SPEED;
                       player_transform.translation.y += direction_vector.y * TIME_STEP * BASE_SPEED;
                   }
               }
            }
            CollisionEvent::DamageEnemy => {
                todo!();
                //if let Ok(mut enemy_health) = enemy_health_query.get_single_mut() {
                //    enemy_health.take_damage(1);
                //}
               
            }
            CollisionEvent::EnemyVsEnemy => {
                if let Ok((mut enemy_collider, mut enemy_transform)) = enemy_query.get_single_mut() {
                    let mut direction_vector = Vec3::ZERO;

                    for collision in enemy_collider.collisions.iter() {
                        let other_enemy_transform = transforms.get(*collision).expect("Collided with entity without collider");

                        let direction = (enemy_transform.translation - other_enemy_transform.translation).normalize();

                        direction_vector += direction;
                    }

                    enemy_collider.collisions.clear();

                    enemy_transform.translation = direction_vector;
                }
            }
        }    
    }
}

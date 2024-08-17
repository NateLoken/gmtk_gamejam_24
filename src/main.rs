mod components;
mod systems;
mod events;

use bevy::prelude::*;
use components::*;
use systems::*;
use events::*;

fn main() {
    App::new()
        .insert_resource(Score::new())
        .insert_resource(MousePosition::default())
        .insert_resource(Points::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                sprite_movement,
                pathfind_towards_player,
                move_entities,
                display_score,
                check_collisions,
                handle_collisions,
                spawn_enemies_over_time,
                camera_follow_player,
                update_mouse_position,
                update_player_position,
                update_lifetime,
                update_cooldowns, // Manage ability cooldowns
                use_ability, // Use abilities including ranged_attack
                manage_invulnerability
            ),
        )
        .add_event::<CollisionEvent>()
        .run();
}

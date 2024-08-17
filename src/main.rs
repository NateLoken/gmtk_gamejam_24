mod components;
mod player;
mod systems;
mod events;

use bevy::prelude::*;
use player::PlayerPlugin;
use systems::*;
use events::*;

//Assets constants
const PLAYER_SPRITE: &str = "blue_box.png";
const ENEMY_SPRITE: &str = "pink_box.png";
const SPRITE_SIZE: (f32, f32) = (225., 225.);
const SPRITE_SCALE: f32 = 0.5;

// Game Cosntants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;

// Texture Resource
#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    enemy: Handle<Image>,
}

fn main() {
    App::new()
        .insert_resource(Score::new())
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                spawn_enemies_over_time,
                pathfind_towards_player,
                move_entities,
                display_score,
                check_collisions,
                handle_collisions,
            ),
        )
        .add_event::<CollisionEvent>()
        .run();
}

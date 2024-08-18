mod components;
mod systems;
mod events;

use bevy::prelude::*;
use bevy_quickmenu::*;
use components::*;
use systems::*;
use events::*;
use components::GameState;


fn main() {
    App::new()
        .insert_resource(Score::new())
        .insert_resource(MousePosition::default())
        .insert_resource(Points::default())
        .insert_resource(CurrentGameState { state: GameState::Running }) 
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, (setup, setup_pause_menu))
        //.add_plugin(QuickMenuPlugin::<PauseMenu>::new()) // Add the QuickMenu plugin
        .add_systems(
            FixedUpdate,
            (
                sprite_movement.run_if(in_state(GameState::Running)),
                pathfind_towards_player.run_if(in_state(GameState::Running)),
                move_entities.run_if(in_state(GameState::Running)),
                display_score.run_if(in_state(GameState::Running)),
                check_collisions.run_if(in_state(GameState::Running)),
                handle_collisions.run_if(in_state(GameState::Running)),
                spawn_enemies_over_time.run_if(in_state(GameState::Running)),
                camera_follow_player.run_if(in_state(GameState::Running)),
                update_mouse_position.run_if(in_state(GameState::Running)),
                update_player_position.run_if(in_state(GameState::Running)),
                update_lifetime.run_if(in_state(GameState::Running)),
                update_cooldowns.run_if(in_state(GameState::Running)),
                use_ability.before(update_cooldowns_ui).run_if(in_state(GameState::Running)),
                update_cooldowns_ui.run_if(in_state(GameState::Running)),
                update_ui_text.run_if(in_state(GameState::Running)),
                manage_invulnerability.run_if(in_state(GameState::Running)),
                flicker_system.run_if(in_state(GameState::Running)),
                handle_escape_pressed.run_if(in_state(GameState::Running).or_else(in_state(GameState::Paused))),
            ))
        .add_event::<CollisionEvent>()
        .add_systems(OnEnter(GameState::Paused), show_pause_menu)
        .add_systems(OnExit(GameState::Paused), hide_pause_menu)
        .run();
}

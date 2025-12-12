// TODO:
// [] Animation Pipline
//  []

mod player;

use bevy::prelude::*;
use std::time::Duration;

use crate::{
    GameState,
    core::player::{MovementState, Player, PlayerSystem, PlayerTextures},
};

// Resources

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(
            Duration::from_secs_f32(1.0 / (fps as f32)),
            TimerMode::Repeating,
        )
    }
}

// Setup
pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerSystem);
        app.add_systems(
            Update,
            (animation_system,).run_if(in_state(GameState::Game)),
        );
    }
}

// Systems
fn animation_system(
    time: Res<Time>,
    textures: Res<PlayerTextures>,
    mut query: Query<(&mut MovementState, &mut AnimationConfig, &mut Sprite), With<Player>>,
) {
    if let Ok((mut state, mut config, mut sprite)) = query.single_mut() {
        if !state.is_moving && !state.is_attacking {
            match state.dir {
                Direction::Left => sprite.image = textures.idle_left.clone(),
                Direction::Right => sprite.image = textures.idle_right.clone(),
                Direction::Up => sprite.image = textures.idle_up.clone(),
                Direction::Down => sprite.image = textures.idle_down.clone(),
            }

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = config.first_sprite_index;
            }
            config.frame_timer.pause();
            return;
        }

        config.frame_timer.unpause();
        config.frame_timer.tick(time.delta());

        if config.frame_timer.is_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index >= config.last_sprite_index {
                atlas.index = config.first_sprite_index;
                if state.is_attacking {
                    state.is_attacking = false;
                }
            } else {
                atlas.index += 1;
            }
        }
    }
}

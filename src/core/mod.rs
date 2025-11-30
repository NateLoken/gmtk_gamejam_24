use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use std::time::Duration;

use crate::GameState;

const SPRITE_SIZE: f32 = 32.0;

// Resources
#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
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
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

// Setup
pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), spawn_player);
        app.add_systems(Update, execute_animations);
        app.add_systems(
            Update,
            trigger_animation::<Player>.run_if(input_just_pressed(KeyCode::ArrowLeft)),
        );
    }
}

// Systems
fn trigger_animation<S: Component>(mut animation: Single<&mut AnimationConfig, With<S>>) {
    animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
}

fn execute_animations(
    time: Res<Time>,
    mut sprite_query: Query<(&mut AnimationConfig, &mut Sprite)>,
) {
    for (mut config, mut sprite) in &mut sprite_query {
        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index == config.last_sprite_index {
                atlas.index = config.first_sprite_index;
            } else {
                atlas.index += 1;
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let player_texture = asset_server.load("RUN/run_left.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        8,
        1,
        Some(uvec2(64, 0)),
        Some(uvec2(32, 26)),
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let run_left = AnimationConfig::new(0, 7, 24);

    commands.spawn((
        Sprite {
            image: player_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: run_left.first_sprite_index,
            }),
            custom_size: Some(Vec2::splat(SPRITE_SIZE / 3.0)),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(7.0)),
        Player,
        run_left,
    ));
}

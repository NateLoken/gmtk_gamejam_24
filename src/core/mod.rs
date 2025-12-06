// TODO:
// [] Animation Pipline
//  []

use bevy::prelude::*;
use std::time::Duration;

use crate::GameState;

const SPRITE_SIZE: f32 = 32.0;
const BASE_SPEED: f32 = 250.0;

// Resources

#[derive(Resource)]
struct PlayerTextures {
    idle_up: Handle<Image>,
    idle_right: Handle<Image>,
    idle_left: Handle<Image>,
    idle_down: Handle<Image>,
    run_up: Handle<Image>,
    run_right: Handle<Image>,
    run_left: Handle<Image>,
    run_down: Handle<Image>,
}

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

#[derive(Component)]
struct MovementState {
    is_moving: bool,
    dir: Direction,
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

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

// Setup
pub struct GameSystems;

impl Plugin for GameSystems {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), spawn_player);
        app.add_systems(
            Update,
            (
                keyboard_input_event_handler,
                movement_system,
                animation_system,
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}

// Systems
fn keyboard_input_event_handler(
    keypress: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut MovementState, &mut Sprite), With<Player>>,
    textures: Res<PlayerTextures>,
) {
    if let Ok((mut vel, mut state, mut sprite)) = query.single_mut() {
        let mut v = Vec2::ZERO;
        let mut dir = state.dir;

        if keypress.pressed(KeyCode::KeyD) {
            v.x = 1.0;
            dir = Direction::Right;
            sprite.image = textures.run_right.clone();
        }
        if keypress.pressed(KeyCode::KeyA) {
            v.x = -1.0;
            dir = Direction::Left;
            sprite.image = textures.run_left.clone();
        }
        if keypress.pressed(KeyCode::KeyW) {
            v.y = 1.0;
            dir = Direction::Up;
            sprite.image = textures.run_up.clone();
        }
        if keypress.pressed(KeyCode::KeyS) {
            v.y = -1.0;
            dir = Direction::Down;
            sprite.image = textures.run_down.clone();
        }

        state.is_moving = v.length_squared() > 0.0;
        state.dir = dir;
        vel.0 = v.normalize_or_zero();
    }
}

fn movement_system(mut query: Query<(&Velocity, &mut Transform), With<Player>>, time: Res<Time>) {
    if let Ok((vel, mut transform)) = query.single_mut() {
        transform.translation.x += (vel.0.x * BASE_SPEED) * time.delta_secs();
        transform.translation.y += (vel.0.y * BASE_SPEED) * time.delta_secs();
    }
}

fn animation_system(
    time: Res<Time>,
    textures: Res<PlayerTextures>,
    mut query: Query<(&MovementState, &mut AnimationConfig, &mut Sprite), With<Player>>,
) {
    if let Ok((state, mut config, mut sprite)) = query.single_mut() {
        if !state.is_moving {
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
            } else {
                atlas.index += 1;
            }
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let player_textures = PlayerTextures {
        idle_up: asset_server.load("IDLE/idle_up.png"),
        idle_left: asset_server.load("IDLE/idle_left.png"),
        idle_right: asset_server.load("IDLE/idle_right.png"),
        idle_down: asset_server.load("IDLE/idle_down.png"),
        run_up: asset_server.load("RUN/run_up.png"),
        run_left: asset_server.load("RUN/run_left.png"),
        run_right: asset_server.load("RUN/run_right.png"),
        run_down: asset_server.load("RUN/run_down.png"),
    };

    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(SPRITE_SIZE as u32),
        8,
        1,
        Some(uvec2(64, 0)),
        Some(uvec2(32, 26)),
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_config = AnimationConfig::new(0, 7, 24);

    commands.spawn((
        Sprite {
            image: player_textures.idle_down.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config.first_sprite_index,
            }),
            custom_size: Some(Vec2::splat(SPRITE_SIZE / 3.0)),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(7.0)),
        Player,
        Velocity(Vec2::ZERO),
        MovementState {
            is_moving: false,
            dir: Direction::Up,
        },
        animation_config,
    ));

    commands.insert_resource(player_textures);
}

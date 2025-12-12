use bevy::prelude::*;

use crate::{
    GameState,
    core::{AnimationConfig, Direction},
};

const SPRITE_SIZE: f32 = 80.0;
const BASE_SPEED: f32 = 300.0;

// Resource
#[derive(Resource)]
pub struct PlayerTextures {
    pub idle_up: Handle<Image>,
    pub idle_right: Handle<Image>,
    pub idle_left: Handle<Image>,
    pub idle_down: Handle<Image>,
    run_up: Handle<Image>,
    run_right: Handle<Image>,
    run_left: Handle<Image>,
    run_down: Handle<Image>,
    attack_up: Handle<Image>,
    attack_right: Handle<Image>,
    attack_left: Handle<Image>,
    attack_down: Handle<Image>,
}

#[derive(Component)]
pub struct MovementState {
    pub is_moving: bool,
    pub is_attacking: bool,
    pub dir: Direction,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity(Vec2);

// Plugin
pub struct PlayerSystem;

impl Plugin for PlayerSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), spawn_player);
        app.add_systems(
            Update,
            (keyboard_input_event_handler, movement_system)
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}

// Systems
pub fn keyboard_input_event_handler(
    keypress: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut MovementState, &mut Sprite), With<Player>>,
    textures: Res<PlayerTextures>,
) {
    if let Ok((mut vel, mut state, mut sprite)) = query.single_mut() {
        let mut v = Vec2::ZERO;
        let mut dir = state.dir;

        if !state.is_attacking {
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
            if keypress.pressed(KeyCode::Space) {
                match state.dir {
                    Direction::Up => sprite.image = textures.attack_up.clone(),
                    Direction::Down => sprite.image = textures.attack_down.clone(),
                    Direction::Left => sprite.image = textures.attack_left.clone(),
                    Direction::Right => sprite.image = textures.attack_right.clone(),
                }
                state.is_attacking = true;
            }
        }

        state.is_moving = v.length_squared() > 0.0;
        state.dir = dir;
        vel.0 = v.normalize_or_zero();
    }
}

pub fn movement_system(
    mut query: Query<(&Velocity, &mut Transform), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((vel, mut transform)) = query.single_mut() {
        transform.translation.x += (vel.0.x * BASE_SPEED) * time.delta_secs();
        transform.translation.y += (vel.0.y * BASE_SPEED) * time.delta_secs();
    }
}

pub fn spawn_player(
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
        attack_up: asset_server.load("ATTACK_1/attack1_up.png"),
        attack_left: asset_server.load("ATTACK_1/attack1_left.png"),
        attack_right: asset_server.load("ATTACK_1/attack1_right.png"),
        attack_down: asset_server.load("ATTACK_1/attack1_down.png"),
    };

    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(SPRITE_SIZE as u32),
        8,
        1,
        Some(uvec2(16, 0)),
        Some(uvec2(7, 0)),
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
            is_attacking: false,
            dir: Direction::Up,
        },
        animation_config,
    ));

    commands.insert_resource(player_textures);
}

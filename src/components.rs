use bevy::prelude::*;

#[derive(Component, PartialEq)]
pub enum Direction {
    Up,
    UpLeft,
    Left,
    Right,
    UpRight,
    Down,
    DownLeft,
    DownRight,
    None,
}

#[derive(Component)]
pub struct Player {
    pub health: i32,
}

impl Player {
    pub fn new(health: i32) -> Self {
        Player { health }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        println!("Player took {} damage, remaining health: {}", amount, self.health);
    }
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct DirectionComponent {
    pub direction: Direction,
}

#[derive(Component)]
pub struct Tag {
    pub name: String,
}

#[derive(Component)]
pub struct CollisionBox {
    pub width: f32,
    pub height: f32,
}

impl CollisionBox {
    pub fn new(width: f32, height: f32) -> Self {
        CollisionBox { width, height }
    }

    pub fn intersects(&self, transform: &Transform, other: &CollisionBox, other_transform: &Transform) -> bool {
        let self_min_x = transform.translation.x - self.width / 2.0;
        let self_max_x = transform.translation.x + self.width / 2.0;
        let self_min_y = transform.translation.y - self.height / 2.0;
        let self_max_y = transform.translation.y + self.height / 2.0;

        let other_min_x = other_transform.translation.x - other.width / 2.0;
        let other_max_x = other_transform.translation.x + other.width / 2.0;
        let other_min_y = other_transform.translation.y - other.height / 2.0;
        let other_max_y = other_transform.translation.y + other.height / 2.0;

        self_min_x < other_max_x &&
        self_max_x > other_min_x &&
        self_min_y < other_max_y &&
        self_max_y > other_min_y
    }
}

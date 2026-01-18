use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default)]
pub struct Grounded(pub bool);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[repr(u8)]
pub enum BlockType {
    Grass,
    Dirt,
    Stone,
    Wood,
    Leaves,
}

#[derive(Component)]
pub struct Mob;

#[derive(Component)]
pub struct Passive;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Hunger(pub f32);

#[derive(Component)]
pub struct Stamina(pub f32);

#[derive(Component)]
pub struct BlockMarker(pub IVec3);

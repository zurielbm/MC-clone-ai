use crate::components::BlockType;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct VoxelWorld {
    pub blocks: HashMap<IVec3, BlockType>,
    pub entities: HashMap<IVec3, Entity>,
}

#[derive(Resource, Default)]
pub struct MaterialHandles {
    pub grass: Handle<StandardMaterial>,
    pub dirt: Handle<StandardMaterial>,
    pub stone: Handle<StandardMaterial>,
    pub wood: Handle<StandardMaterial>,
    pub leaves: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct TimeOfDay(pub f32); // 0.0 to 1.0 (normalized day)

#[derive(Resource, Default)]
pub struct Inventory {
    pub items: HashMap<crate::components::BlockType, u32>,
}

#[derive(Resource, Default)]
pub struct CubeMesh(pub Handle<Mesh>);

#[derive(Resource)]
pub struct SelectedBlock(pub crate::components::BlockType);

#[derive(Event)]
pub struct RaycastHit {
    pub coord: IVec3,
    pub normal: IVec3,
    pub entity: Option<Entity>,
}

#[derive(Event)]
pub struct HungerDepleted;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    InGame,
    Paused,
    GameOver,
}

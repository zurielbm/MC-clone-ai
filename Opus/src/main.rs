use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::DistanceFog;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use std::collections::HashMap;
use std::f32::consts::PI;

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Grounded(bool);

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
enum BlockType {
    Grass = 0,
    Dirt = 1,
    Stone = 2,
    Wood = 3,
    Leaves = 4,
}

#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct MaxHealth(f32);

#[derive(Component)]
struct Hunger(f32);

#[derive(Component)]
struct Stamina(f32);

#[derive(Component)]
struct Block;

// Player dimensions for collision
#[derive(Component)]
struct PlayerAABB {
    half_width: f32,
    half_height: f32,
}

impl Default for PlayerAABB {
    fn default() -> Self {
        Self {
            half_width: 0.3,
            half_height: 0.9,
        }
    }
}

// UI marker components
#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct HungerBar;

#[derive(Component)]
struct StaminaBar;

#[derive(Component)]
struct HotbarSlot(usize);

#[derive(Component)]
struct HotbarItemIcon(usize);

#[derive(Component)]
struct HotbarSelector;

#[derive(Component)]
struct InventoryUI;

#[derive(Component)]
struct CraftingUI;

#[derive(Component)]
struct CraftingSlot {
    row: usize,
    col: usize,
}

#[derive(Component)]
struct CraftingOutput;

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct SelectedItemName;

#[derive(Component)]
struct PauseMenu;

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct QuitButton;

#[derive(Resource)]
struct SelectedItemTimer(f32);

impl Default for SelectedItemTimer {
    fn default() -> Self {
        Self(0.0)
    }
}

// Mob components
#[derive(Component)]
struct Mob;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MobType {
    Pig,
    Sheep,
    Zombie,
}

#[derive(Component)]
struct MobAI {
    state: AIState,
    target: Option<Entity>,
    timer: f32,
    direction: Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AIState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
}

#[derive(Component)]
struct MobHealthBar;

#[derive(Component)]
struct MobHealthBarFill;

// Hit feedback
#[derive(Component)]
struct HitFlash {
    timer: f32,
    original_color: Color,
}

#[derive(Component)]
struct DamageNumber {
    timer: f32,
    velocity: Vec3,
}

// Mob animation
#[derive(Component)]
struct MobAnimation {
    time: f32,
    is_moving: bool,
}

#[derive(Component)]
struct MobLeg {
    is_front: bool,
    is_left: bool,
}

// Day/Night cycle
#[derive(Resource)]
struct DayNightCycle {
    time: f32, // 0.0 to 1.0 (0 = midnight, 0.25 = sunrise, 0.5 = noon, 0.75 = sunset)
    day_length_seconds: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time: 0.35,                // Start at morning
            day_length_seconds: 120.0, // 2 minute day cycle
        }
    }
}

impl DayNightCycle {
    fn sun_intensity(&self) -> f32 {
        // Brightest at noon (0.5), darkest at midnight (0.0)
        let t = (self.time - 0.25).abs();
        if t < 0.25 {
            1.0 - (t * 4.0) * 0.7 // Day: 1.0 to 0.3
        } else {
            0.1 + ((t - 0.25) * 4.0).min(1.0) * 0.2 // Night: 0.1 to 0.3
        }
    }

    fn sky_color(&self) -> Color {
        if self.time > 0.2 && self.time < 0.8 {
            // Day
            Color::srgb(0.5, 0.7, 1.0)
        } else if self.time > 0.75 || self.time < 0.05 {
            // Night
            Color::srgb(0.05, 0.05, 0.15)
        } else if self.time < 0.2 {
            // Sunrise
            let t = self.time / 0.2;
            Color::srgb(0.3 + t * 0.2, 0.2 + t * 0.5, 0.3 + t * 0.7)
        } else {
            // Sunset
            let t = (self.time - 0.75) / 0.05;
            Color::srgb(0.5 - t * 0.45, 0.3 - t * 0.25, 0.3 - t * 0.15)
        }
    }

    fn ambient_color(&self) -> Color {
        if self.time > 0.25 && self.time < 0.75 {
            Color::srgb(0.6, 0.7, 1.0)
        } else {
            Color::srgb(0.1, 0.1, 0.3)
        }
    }
}

#[derive(Component)]
struct Sun;

// Dropped items
#[derive(Component)]
struct DroppedItem {
    item_type: ItemType,
    count: u32,
}

#[derive(Component)]
struct ItemBob {
    base_y: f32,
    time: f32,
}

// ============================================================================
// ITEM TYPES
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum ItemType {
    Block(BlockType),
    RawPork,
    Wool,
    RottenFlesh,
    Stick,
    WoodPickaxe,
}

impl ItemType {
    fn max_stack(&self) -> u32 {
        match self {
            ItemType::WoodPickaxe => 1,
            _ => 64,
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            ItemType::Block(BlockType::Grass) => "Grass",
            ItemType::Block(BlockType::Dirt) => "Dirt",
            ItemType::Block(BlockType::Stone) => "Stone",
            ItemType::Block(BlockType::Wood) => "Wood",
            ItemType::Block(BlockType::Leaves) => "Leaves",
            ItemType::RawPork => "Raw Pork",
            ItemType::Wool => "Wool",
            ItemType::RottenFlesh => "Rotten Flesh",
            ItemType::Stick => "Stick",
            ItemType::WoodPickaxe => "Wood Pickaxe",
        }
    }

    fn color(&self) -> Color {
        match self {
            ItemType::Block(BlockType::Grass) => Color::srgb(0.2, 0.7, 0.2),
            ItemType::Block(BlockType::Dirt) => Color::srgb(0.5, 0.35, 0.2),
            ItemType::Block(BlockType::Stone) => Color::srgb(0.5, 0.5, 0.5),
            ItemType::Block(BlockType::Wood) => Color::srgb(0.6, 0.4, 0.2),
            ItemType::Block(BlockType::Leaves) => Color::srgb(0.1, 0.5, 0.1),
            ItemType::RawPork => Color::srgb(1.0, 0.6, 0.6),
            ItemType::Wool => Color::srgb(0.95, 0.95, 0.95),
            ItemType::RottenFlesh => Color::srgb(0.5, 0.4, 0.3),
            ItemType::Stick => Color::srgb(0.7, 0.5, 0.3),
            ItemType::WoodPickaxe => Color::srgb(0.8, 0.6, 0.4),
        }
    }
}

#[derive(Clone, Copy)]
struct ItemStack {
    item_type: ItemType,
    count: u32,
}

// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource)]
struct VoxelWorld {
    blocks: HashMap<IVec3, (BlockType, Entity)>,
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self {
            blocks: HashMap::with_capacity(4096),
        }
    }
}

#[derive(Resource)]
struct MaterialHandles {
    materials: [Handle<StandardMaterial>; 5],
}

#[derive(Resource)]
struct MobMaterials {
    pig: Handle<StandardMaterial>,
    sheep: Handle<StandardMaterial>,
    zombie: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct CubeMesh(Handle<Mesh>);

#[derive(Resource)]
struct Inventory {
    slots: [Option<ItemStack>; 36],
    selected_slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        let mut slots = [None; 36];
        // Start with some dirt blocks
        slots[0] = Some(ItemStack {
            item_type: ItemType::Block(BlockType::Dirt),
            count: 64,
        });
        slots[1] = Some(ItemStack {
            item_type: ItemType::Block(BlockType::Stone),
            count: 64,
        });
        slots[2] = Some(ItemStack {
            item_type: ItemType::Block(BlockType::Wood),
            count: 32,
        });
        Self {
            slots,
            selected_slot: 0,
        }
    }
}

impl Inventory {
    fn add_item(&mut self, item_type: ItemType, mut count: u32) -> bool {
        // First try to stack with existing
        for slot in self.slots.iter_mut() {
            if count == 0 {
                break;
            }
            if let Some(stack) = slot {
                if stack.item_type == item_type {
                    let can_add = (item_type.max_stack() - stack.count).min(count);
                    stack.count += can_add;
                    count -= can_add;
                }
            }
        }
        // Then try empty slots
        for slot in self.slots.iter_mut() {
            if count == 0 {
                break;
            }
            if slot.is_none() {
                let add_count = count.min(item_type.max_stack());
                *slot = Some(ItemStack {
                    item_type,
                    count: add_count,
                });
                count -= add_count;
            }
        }
        count == 0
    }

    fn remove_selected(&mut self) -> bool {
        if let Some(stack) = &mut self.slots[self.selected_slot] {
            stack.count -= 1;
            if stack.count == 0 {
                self.slots[self.selected_slot] = None;
            }
            true
        } else {
            false
        }
    }
}

#[derive(Resource)]
struct CraftingGrid {
    slots: [[Option<ItemStack>; 3]; 3],
}

impl Default for CraftingGrid {
    fn default() -> Self {
        Self {
            slots: [[None; 3]; 3],
        }
    }
}

#[derive(Resource)]
struct CraftingRecipes(Vec<Recipe>);

struct Recipe {
    pattern: [[Option<ItemType>; 3]; 3],
    output: ItemStack,
}

impl Default for CraftingRecipes {
    fn default() -> Self {
        Self(vec![
            // Wood Log -> 4 Planks (simplified: just wood in center)
            Recipe {
                pattern: [
                    [None, None, None],
                    [None, Some(ItemType::Block(BlockType::Wood)), None],
                    [None, None, None],
                ],
                output: ItemStack {
                    item_type: ItemType::Block(BlockType::Dirt),
                    count: 4,
                }, // Planks as dirt for now
            },
            // 2 Wood -> 4 Sticks
            Recipe {
                pattern: [
                    [None, Some(ItemType::Block(BlockType::Wood)), None],
                    [None, Some(ItemType::Block(BlockType::Wood)), None],
                    [None, None, None],
                ],
                output: ItemStack {
                    item_type: ItemType::Stick,
                    count: 4,
                },
            },
        ])
    }
}

#[derive(Resource, Default)]
struct GameUI {
    inventory_open: bool,
    crafting_open: bool,
    paused: bool,
}

#[derive(Resource)]
struct ItemDropAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

// ============================================================================
// EVENTS
// ============================================================================

#[derive(Event)]
struct RaycastHit {
    coord: IVec3,
    normal: IVec3,
}

#[derive(Event)]
struct HungerDepleted;

#[derive(Event)]
struct MobHit {
    entity: Entity,
    damage: f32,
}

// ============================================================================
// CONSTANTS
// ============================================================================

const GRAVITY: f32 = -25.0;
const JUMP_VELOCITY: f32 = 9.0;
const MOVE_SPEED: f32 = 6.0;
const MOUSE_SENSITIVITY: f32 = 0.003;
const HUNGER_DECAY_RATE: f32 = 0.05;
const STARVATION_DAMAGE: f32 = 5.0;
const PLAYER_ATTACK_DAMAGE: f32 = 5.0;
const ZOMBIE_ATTACK_DAMAGE: f32 = 2.0;
const ZOMBIE_ATTACK_RANGE: f32 = 1.5;
const ZOMBIE_DETECT_RANGE: f32 = 16.0;
const ITEM_PICKUP_RANGE: f32 = 2.0;

// ============================================================================
// STARTUP SYSTEMS
// ============================================================================

fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create cube mesh
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands.insert_resource(CubeMesh(cube_mesh));

    // Create materials for each block type
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.7, 0.2),
        perceptual_roughness: 0.9,
        ..default()
    });

    let dirt_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.35, 0.2),
        perceptual_roughness: 0.9,
        ..default()
    });

    let stone_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.8,
        ..default()
    });

    let wood_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2),
        perceptual_roughness: 0.9,
        ..default()
    });

    let leaves_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0.5, 0.1, 0.9),
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(MaterialHandles {
        materials: [
            grass_material,
            dirt_material,
            stone_material,
            wood_material,
            leaves_material,
        ],
    });

    // Mob materials
    let pig_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.75, 0.7),
        perceptual_roughness: 0.8,
        ..default()
    });

    let sheep_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.95),
        perceptual_roughness: 0.9,
        ..default()
    });

    let zombie_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.6, 0.4),
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.insert_resource(MobMaterials {
        pig: pig_material,
        sheep: sheep_material,
        zombie: zombie_material,
    });

    // Add directional light (sun)
    commands.spawn((
        Sun,
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.7, 1.0),
        brightness: 500.0,
    });

    // Clear color (sky)
    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0)));

    // Item drop assets (cached to prevent lag on attack)
    let item_drop_mesh = meshes.add(Cuboid::new(0.3, 0.3, 0.3));
    let item_drop_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.8, 0.6),
        ..default()
    });
    commands.insert_resource(ItemDropAssets {
        mesh: item_drop_mesh,
        material: item_drop_material,
    });
}

fn setup_world(
    mut commands: Commands,
    cube_mesh: Res<CubeMesh>,
    material_handles: Res<MaterialHandles>,
    mut voxel_world: ResMut<VoxelWorld>,
) {
    // Spawn larger terrain (32x32x4)
    for x in -16..16 {
        for z in -16..16 {
            for y in 0..4 {
                let block_type = if y == 3 {
                    BlockType::Grass
                } else if y >= 1 {
                    BlockType::Dirt
                } else {
                    BlockType::Stone
                };

                let coord = IVec3::new(x, y, z);
                let material = material_handles.materials[block_type as usize].clone();

                let entity = commands
                    .spawn((
                        Mesh3d(cube_mesh.0.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(coord.as_vec3()),
                        block_type,
                        Block,
                    ))
                    .id();

                voxel_world.blocks.insert(coord, (block_type, entity));
            }
        }
    }

    // Spawn trees
    let tree_positions = [
        IVec3::new(5, 4, 5),
        IVec3::new(-8, 4, 3),
        IVec3::new(10, 4, -6),
        IVec3::new(-5, 4, -10),
        IVec3::new(8, 4, 12),
        IVec3::new(-12, 4, 8),
        IVec3::new(3, 4, -12),
    ];

    for base in tree_positions {
        spawn_tree(
            &mut commands,
            &cube_mesh,
            &material_handles,
            &mut voxel_world,
            base,
        );
    }
}

fn spawn_tree(
    commands: &mut Commands,
    cube_mesh: &Res<CubeMesh>,
    material_handles: &Res<MaterialHandles>,
    voxel_world: &mut ResMut<VoxelWorld>,
    base: IVec3,
) {
    // Trunk (4-6 blocks tall)
    let trunk_height = 5;
    for y in 0..trunk_height {
        let coord = base + IVec3::new(0, y, 0);
        if voxel_world.blocks.contains_key(&coord) {
            continue;
        }

        let entity = commands
            .spawn((
                Mesh3d(cube_mesh.0.clone()),
                MeshMaterial3d(material_handles.materials[BlockType::Wood as usize].clone()),
                Transform::from_translation(coord.as_vec3()),
                BlockType::Wood,
                Block,
            ))
            .id();
        voxel_world.blocks.insert(coord, (BlockType::Wood, entity));
    }

    // Leaves (3x3x3 canopy at top)
    let leaf_base = base + IVec3::new(0, trunk_height - 1, 0);
    for dx in -1_i32..=1 {
        for dy in 0_i32..=2 {
            for dz in -1_i32..=1 {
                // Skip corners on bottom and top layers for more natural look
                if (dy == 0 || dy == 2) && dx.abs() == 1 && dz.abs() == 1 {
                    continue;
                }
                // Skip center column where trunk is (except top)
                if dx == 0 && dz == 0 && dy < 2 {
                    continue;
                }

                let coord = leaf_base + IVec3::new(dx, dy, dz);
                if voxel_world.blocks.contains_key(&coord) {
                    continue;
                }

                let entity = commands
                    .spawn((
                        Mesh3d(cube_mesh.0.clone()),
                        MeshMaterial3d(
                            material_handles.materials[BlockType::Leaves as usize].clone(),
                        ),
                        Transform::from_translation(coord.as_vec3()),
                        BlockType::Leaves,
                        Block,
                    ))
                    .id();
                voxel_world
                    .blocks
                    .insert(coord, (BlockType::Leaves, entity));
            }
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            Transform::from_xyz(0.0, 6.0, 0.0),
            Visibility::default(),
            Velocity(Vec3::ZERO),
            Grounded(false),
            PlayerAABB::default(),
            Health(100.0),
            MaxHealth(100.0),
            Hunger(100.0),
            Stamina(100.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                MainCamera,
                Transform::from_xyz(0.0, 0.6, 0.0),
                DistanceFog {
                    color: Color::srgba(0.6, 0.75, 1.0, 1.0),
                    falloff: FogFalloff::Linear {
                        start: 30.0,
                        end: 80.0,
                    },
                    ..default()
                },
            ));
        });
}

fn spawn_mobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mob_materials: Res<MobMaterials>,
) {
    // Create mesh parts for mobs
    let body_mesh_pig = meshes.add(Cuboid::new(0.8, 0.5, 0.5));
    let head_mesh_pig = meshes.add(Cuboid::new(0.4, 0.4, 0.35));
    let snout_mesh = meshes.add(Cuboid::new(0.2, 0.15, 0.1));
    let leg_mesh = meshes.add(Cuboid::new(0.15, 0.3, 0.15));

    let body_mesh_sheep = meshes.add(Cuboid::new(0.9, 0.6, 0.6));
    let head_mesh_sheep = meshes.add(Cuboid::new(0.35, 0.35, 0.3));

    let body_mesh_zombie = meshes.add(Cuboid::new(0.5, 0.7, 0.3));
    let head_mesh_zombie = meshes.add(Cuboid::new(0.4, 0.4, 0.4));
    let arm_mesh = meshes.add(Cuboid::new(0.15, 0.5, 0.15));
    let leg_mesh_zombie = meshes.add(Cuboid::new(0.18, 0.5, 0.18));

    // Spawn passive mobs (pigs and sheep)
    let passive_positions = [
        (Vec3::new(8.0, 4.0, 8.0), MobType::Pig),
        (Vec3::new(-6.0, 4.0, 10.0), MobType::Sheep),
        (Vec3::new(12.0, 4.0, -4.0), MobType::Pig),
        (Vec3::new(-10.0, 4.0, -8.0), MobType::Sheep),
    ];

    for (pos, mob_type) in passive_positions {
        match mob_type {
            MobType::Pig => spawn_pig(
                &mut commands,
                &body_mesh_pig,
                &head_mesh_pig,
                &snout_mesh,
                &leg_mesh,
                &mob_materials.pig,
                pos,
            ),
            MobType::Sheep => spawn_sheep(
                &mut commands,
                &body_mesh_sheep,
                &head_mesh_sheep,
                &leg_mesh,
                &mob_materials.sheep,
                pos,
            ),
            MobType::Zombie => {}
        }
    }

    // Spawn hostile mobs (zombies)
    let zombie_positions = [Vec3::new(-12.0, 4.0, 12.0), Vec3::new(14.0, 4.0, 10.0)];

    for pos in zombie_positions {
        spawn_zombie(
            &mut commands,
            &body_mesh_zombie,
            &head_mesh_zombie,
            &arm_mesh,
            &leg_mesh_zombie,
            &mob_materials.zombie,
            pos,
        );
    }
}

fn spawn_pig(
    commands: &mut Commands,
    body_mesh: &Handle<Mesh>,
    head_mesh: &Handle<Mesh>,
    snout_mesh: &Handle<Mesh>,
    leg_mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
) {
    commands
        .spawn((
            Mob,
            MobType::Pig,
            Transform::from_translation(position),
            Visibility::default(),
            Velocity(Vec3::ZERO),
            Health(20.0),
            MaxHealth(20.0),
            MobAnimation {
                time: fastrand::f32() * 6.28,
                is_moving: false,
            },
            MobAI {
                state: AIState::Idle,
                target: None,
                timer: 0.0,
                direction: Vec3::ZERO,
            },
        ))
        .with_children(|parent| {
            // Body
            parent.spawn((
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.4, 0.0),
            ));
            // Head
            parent.spawn((
                Mesh3d(head_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.5, 0.5, 0.0),
            ));
            // Snout (pink)
            parent.spawn((
                Mesh3d(snout_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.75, 0.45, 0.0),
            ));
            // Legs
            for (x, z) in [(-0.25, -0.15), (-0.25, 0.15), (0.25, -0.15), (0.25, 0.15)] {
                parent.spawn((
                    Mesh3d(leg_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(x, 0.15, z),
                ));
            }
        });
}

fn spawn_sheep(
    commands: &mut Commands,
    body_mesh: &Handle<Mesh>,
    head_mesh: &Handle<Mesh>,
    leg_mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
) {
    commands
        .spawn((
            Mob,
            MobType::Sheep,
            Transform::from_translation(position),
            Visibility::default(),
            Velocity(Vec3::ZERO),
            Health(20.0),
            MaxHealth(20.0),
            MobAnimation {
                time: fastrand::f32() * 6.28,
                is_moving: false,
            },
            MobAI {
                state: AIState::Idle,
                target: None,
                timer: 0.0,
                direction: Vec3::ZERO,
            },
        ))
        .with_children(|parent| {
            // Fluffy body
            parent.spawn((
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.5, 0.0),
            ));
            // Head (darker)
            parent.spawn((
                Mesh3d(head_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.5, 0.55, 0.0),
            ));
            // Legs
            for (x, z) in [(-0.3, -0.2), (-0.3, 0.2), (0.3, -0.2), (0.3, 0.2)] {
                parent.spawn((
                    Mesh3d(leg_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(x, 0.15, z),
                ));
            }
        });
}

fn spawn_zombie(
    commands: &mut Commands,
    body_mesh: &Handle<Mesh>,
    head_mesh: &Handle<Mesh>,
    arm_mesh: &Handle<Mesh>,
    leg_mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
) {
    commands
        .spawn((
            Mob,
            MobType::Zombie,
            Transform::from_translation(position),
            Visibility::default(),
            Velocity(Vec3::ZERO),
            Health(30.0),
            MaxHealth(30.0),
            MobAnimation {
                time: fastrand::f32() * 6.28,
                is_moving: false,
            },
            MobAI {
                state: AIState::Idle,
                target: None,
                timer: 0.0,
                direction: Vec3::ZERO,
            },
        ))
        .with_children(|parent| {
            // Body
            parent.spawn((
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.85, 0.0),
            ));
            // Head
            parent.spawn((
                Mesh3d(head_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 1.4, 0.0),
            ));
            // Arms (stretched forward like zombie)
            parent.spawn((
                Mesh3d(arm_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.35, 1.0, 0.3).with_rotation(Quat::from_rotation_x(-0.5)),
            ));
            parent.spawn((
                Mesh3d(arm_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(-0.35, 1.0, 0.3).with_rotation(Quat::from_rotation_x(-0.5)),
            ));
            // Legs
            parent.spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.15, 0.25, 0.0),
            ));
            parent.spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(-0.15, 0.25, 0.0),
            ));
        });
}

fn setup_ui(mut commands: Commands) {
    // Root UI
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|root| {
            // Top section - survival bars and FPS
            root.spawn(Node {
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|top_row| {
                // Left side - survival bars
                top_row
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|bars| {
                        spawn_stat_bar(bars, "Health", Color::srgb(0.8, 0.2, 0.2), HealthBar);
                        spawn_stat_bar(bars, "Hunger", Color::srgb(0.8, 0.6, 0.2), HungerBar);
                        spawn_stat_bar(bars, "Stamina", Color::srgb(0.2, 0.6, 0.8), StaminaBar);
                    });

                // Right side - FPS counter
                top_row.spawn((
                    Text::new("FPS: --"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 1.0, 0.0)),
                    FpsText,
                ));
            });

            // Bottom section - hotbar and item name
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Px(20.0)),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|bottom| {
                // Selected item name (above hotbar)
                bottom.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    SelectedItemName,
                ));

                // Hotbar container
                bottom
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|hotbar| {
                        for i in 0..9 {
                            hotbar
                                .spawn((
                                    Node {
                                        width: Val::Px(50.0),
                                        height: Val::Px(50.0),
                                        justify_content: JustifyContent::End,
                                        align_items: AlignItems::End,
                                        border: UiRect::all(Val::Px(2.0)),
                                        padding: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                                    BorderColor(if i == 0 {
                                        Color::WHITE
                                    } else {
                                        Color::srgba(0.4, 0.4, 0.4, 0.8)
                                    }),
                                    HotbarSlot(i),
                                ))
                                .with_children(|slot| {
                                    // Item color indicator (colored square)
                                    slot.spawn((
                                        Node {
                                            width: Val::Px(32.0),
                                            height: Val::Px(32.0),
                                            position_type: PositionType::Absolute,
                                            left: Val::Px(7.0),
                                            top: Val::Px(7.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        HotbarItemIcon(i),
                                    ));
                                    // Item count text
                                    slot.spawn((
                                        Text::new(""),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        }
                    });
            });
        });

    // Crosshair
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
        });
}

fn spawn_stat_bar<T: Component>(parent: &mut ChildBuilder, label: &str, color: Color, marker: T) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    width: Val::Px(70.0),
                    ..default()
                },
            ));

            row.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            ))
            .with_children(|bg| {
                bg.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(color),
                    marker,
                ));
            });
        });
}

fn grab_cursor(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

// ============================================================================
// UPDATE SYSTEMS
// ============================================================================

fn player_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    game_ui: Res<GameUI>,
) {
    if game_ui.inventory_open || game_ui.crafting_open || game_ui.paused {
        return;
    }

    let mut delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        delta += motion.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    if let Ok(mut player_transform) = player_query.get_single_mut() {
        player_transform.rotate_y(-delta.x * MOUSE_SENSITIVITY);
    }

    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let pitch = -delta.y * MOUSE_SENSITIVITY;
        let (yaw, current_pitch, roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        let new_pitch = (current_pitch + pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
        camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, new_pitch, roll);
    }
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut Velocity, &Grounded), With<Player>>,
    game_ui: Res<GameUI>,
) {
    let Ok((transform, mut velocity, grounded)) = player_query.get_single_mut() else {
        return;
    };

    // If menu is open, stop horizontal movement but keep gravity
    if game_ui.inventory_open || game_ui.crafting_open || game_ui.paused {
        velocity.0.x = 0.0;
        velocity.0.z = 0.0;
        return;
    }

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= transform.right().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += transform.right().as_vec3();
    }

    direction.y = 0.0;
    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    velocity.0.x = direction.x * MOVE_SPEED;
    velocity.0.z = direction.z * MOVE_SPEED;

    if keyboard.just_pressed(KeyCode::Space) && grounded.0 {
        velocity.0.y = JUMP_VELOCITY;
    }
}

fn hotbar_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut hotbar_slots: Query<(&HotbarSlot, &mut BorderColor)>,
) {
    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    for (i, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            inventory.selected_slot = i;
        }
    }

    // Update visual selection
    for (slot, mut border) in hotbar_slots.iter_mut() {
        border.0 = if slot.0 == inventory.selected_slot {
            Color::WHITE
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.8)
        };
    }
}

fn toggle_menus(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_ui: ResMut<GameUI>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    pause_menu_query: Query<Entity, With<PauseMenu>>,
    crafting_ui_query: Query<Entity, With<CraftingUI>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) && !game_ui.paused {
        game_ui.inventory_open = !game_ui.inventory_open;
        if game_ui.inventory_open {
            game_ui.crafting_open = false;
            // Despawn crafting UI
            for entity in crafting_ui_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
        update_cursor_state(
            &mut windows,
            game_ui.inventory_open || game_ui.crafting_open,
        );
    }

    if keyboard.just_pressed(KeyCode::KeyE) && !game_ui.paused {
        game_ui.crafting_open = !game_ui.crafting_open;
        if game_ui.crafting_open {
            game_ui.inventory_open = false;
            // Spawn crafting UI
            spawn_crafting_ui(&mut commands);
        } else {
            // Despawn crafting UI
            for entity in crafting_ui_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
        update_cursor_state(
            &mut windows,
            game_ui.inventory_open || game_ui.crafting_open,
        );
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        if game_ui.inventory_open || game_ui.crafting_open {
            game_ui.inventory_open = false;
            game_ui.crafting_open = false;
            update_cursor_state(&mut windows, false);
        } else {
            // Toggle pause menu
            game_ui.paused = !game_ui.paused;
            update_cursor_state(&mut windows, game_ui.paused);

            if game_ui.paused {
                // Spawn pause menu
                spawn_pause_menu(&mut commands);
            } else {
                // Despawn pause menu
                for entity in pause_menu_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

fn spawn_pause_menu(commands: &mut Commands) {
    commands
        .spawn((
            PauseMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Menu container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        row_gap: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.95)),
                ))
                .with_children(|menu| {
                    // Title
                    menu.spawn((
                        Text::new("PAUSED"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Resume button
                    menu.spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                        ResumeButton,
                        Button,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Resume"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Quit button
                    menu.spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                        QuitButton,
                        Button,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Quit"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        });
}

fn spawn_crafting_ui(commands: &mut Commands) {
    commands
        .spawn((
            CraftingUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // Main crafting container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(30.0),
                        padding: UiRect::all(Val::Px(30.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.95)),
                ))
                .with_children(|container| {
                    // Left side: 3x3 crafting grid
                    container
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|grid_container| {
                            // Title
                            grid_container.spawn((
                                Text::new("Crafting"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // 3x3 Grid
                            for row in 0..3 {
                                grid_container
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(4.0),
                                        ..default()
                                    })
                                    .with_children(|row_node| {
                                        for col in 0..3 {
                                            row_node.spawn((
                                                Node {
                                                    width: Val::Px(50.0),
                                                    height: Val::Px(50.0),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    border: UiRect::all(Val::Px(2.0)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 0.9)),
                                                BorderColor(Color::srgba(0.5, 0.5, 0.55, 0.9)),
                                                CraftingSlot { row, col },
                                                Button,
                                            ));
                                        }
                                    });
                            }
                        });

                    // Arrow in the middle
                    container.spawn((
                        Text::new("=>"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Right side: output slot
                    container
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|output_container| {
                            output_container.spawn((
                                Text::new("Output"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            output_container.spawn((
                                Node {
                                    width: Val::Px(60.0),
                                    height: Val::Px(60.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.3, 0.5, 0.3, 0.9)),
                                BorderColor(Color::srgb(0.4, 0.6, 0.4)),
                                CraftingOutput,
                                Button,
                            ));
                        });
                });
        });
}

fn handle_pause_buttons(
    mut game_ui: ResMut<GameUI>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    resume_query: Query<&Interaction, (With<ResumeButton>, Changed<Interaction>)>,
    quit_query: Query<&Interaction, (With<QuitButton>, Changed<Interaction>)>,
    pause_menu_query: Query<Entity, With<PauseMenu>>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    for interaction in resume_query.iter() {
        if *interaction == Interaction::Pressed {
            game_ui.paused = false;
            update_cursor_state(&mut windows, false);
            for entity in pause_menu_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    for interaction in quit_query.iter() {
        if *interaction == Interaction::Pressed {
            exit.send(bevy::app::AppExit::Success);
        }
    }
}

fn update_cursor_state(windows: &mut Query<&mut Window, With<PrimaryWindow>>, menu_open: bool) {
    if let Ok(mut window) = windows.get_single_mut() {
        if menu_open {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }
    }
}

// ============================================================================
// PHYSICS SYSTEMS
// ============================================================================

fn apply_physics(
    time: Res<Time>,
    voxel_world: Res<VoxelWorld>,
    mut query: Query<(&mut Transform, &mut Velocity, &PlayerAABB, &mut Grounded), With<Player>>,
) {
    let Ok((mut transform, mut velocity, aabb, mut grounded)) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Apply gravity
    velocity.0.y += GRAVITY * dt;

    // Move in each axis separately for proper collision response
    let new_pos = transform.translation + velocity.0 * dt;

    // X axis
    let test_x = Vec3::new(new_pos.x, transform.translation.y, transform.translation.z);
    if !check_collision(&voxel_world, test_x, aabb) {
        transform.translation.x = new_pos.x;
    } else {
        velocity.0.x = 0.0;
    }

    // Z axis
    let test_z = Vec3::new(transform.translation.x, transform.translation.y, new_pos.z);
    if !check_collision(&voxel_world, test_z, aabb) {
        transform.translation.z = new_pos.z;
    } else {
        velocity.0.z = 0.0;
    }

    // Y axis
    let test_y = Vec3::new(transform.translation.x, new_pos.y, transform.translation.z);
    if !check_collision(&voxel_world, test_y, aabb) {
        transform.translation.y = new_pos.y;
        grounded.0 = false;
    } else {
        if velocity.0.y < 0.0 {
            grounded.0 = true;
            // Snap to top of block
            let feet_y = new_pos.y - aabb.half_height;
            let block_y = feet_y.floor() + 1.0;
            transform.translation.y = block_y + aabb.half_height;
        }
        velocity.0.y = 0.0;
    }
}

fn check_collision(voxel_world: &VoxelWorld, position: Vec3, aabb: &PlayerAABB) -> bool {
    let min = position - Vec3::new(aabb.half_width, aabb.half_height, aabb.half_width);
    let max = position + Vec3::new(aabb.half_width, aabb.half_height, aabb.half_width);

    let min_block = IVec3::new(
        min.x.floor() as i32,
        min.y.floor() as i32,
        min.z.floor() as i32,
    );
    let max_block = IVec3::new(
        max.x.floor() as i32,
        max.y.floor() as i32,
        max.z.floor() as i32,
    );

    for x in min_block.x..=max_block.x {
        for y in min_block.y..=max_block.y {
            for z in min_block.z..=max_block.z {
                if voxel_world.blocks.contains_key(&IVec3::new(x, y, z)) {
                    // Check AABB intersection
                    let block_min = Vec3::new(x as f32, y as f32, z as f32);
                    let block_max = block_min + Vec3::ONE;

                    if min.x < block_max.x
                        && max.x > block_min.x
                        && min.y < block_max.y
                        && max.y > block_min.y
                        && min.z < block_max.z
                        && max.z > block_min.z
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn hunger_decay(
    time: Res<Time>,
    mut query: Query<&mut Hunger, With<Player>>,
    mut hunger_depleted: EventWriter<HungerDepleted>,
) {
    let Ok(mut hunger) = query.get_single_mut() else {
        return;
    };

    hunger.0 -= time.delta_secs() * HUNGER_DECAY_RATE;

    if hunger.0 <= 0.0 {
        hunger.0 = 0.0;
        hunger_depleted.send(HungerDepleted);
    }
}

fn starvation_damage(
    time: Res<Time>,
    mut events: EventReader<HungerDepleted>,
    mut query: Query<&mut Health, With<Player>>,
) {
    if events.read().count() == 0 {
        return;
    }

    let Ok(mut health) = query.get_single_mut() else {
        return;
    };
    health.0 = (health.0 - time.delta_secs() * STARVATION_DAMAGE).max(0.0);
}

// ============================================================================
// MOB AI SYSTEMS
// ============================================================================

fn mob_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut mob_query: Query<(&Transform, &mut MobAI, &mut Velocity, &MobType), With<Mob>>,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    for (transform, mut ai, mut velocity, mob_type) in mob_query.iter_mut() {
        ai.timer -= time.delta_secs();

        match mob_type {
            MobType::Zombie => {
                let dist = transform.translation.distance(player_pos);
                if dist < ZOMBIE_DETECT_RANGE {
                    ai.state = if dist < ZOMBIE_ATTACK_RANGE {
                        AIState::Attacking
                    } else {
                        AIState::Chasing
                    };
                    ai.direction = (player_pos - transform.translation).normalize_or_zero();
                    ai.direction.y = 0.0;
                } else {
                    ai.state = AIState::Wandering;
                }
            }
            _ => {
                // Passive mobs wander
                if ai.timer <= 0.0 {
                    ai.timer = 2.0 + fastrand::f32() * 3.0;
                    if fastrand::f32() < 0.5 {
                        ai.state = AIState::Wandering;
                        let angle = fastrand::f32() * PI * 2.0;
                        ai.direction = Vec3::new(angle.cos(), 0.0, angle.sin());
                    } else {
                        ai.state = AIState::Idle;
                    }
                }
            }
        }

        // Apply movement based on state
        let speed = match ai.state {
            AIState::Idle => 0.0,
            AIState::Wandering => 1.5,
            AIState::Chasing => 3.0,
            AIState::Attacking => 0.0,
        };

        velocity.0.x = ai.direction.x * speed;
        velocity.0.z = ai.direction.z * speed;
    }
}

fn mob_physics(
    time: Res<Time>,
    voxel_world: Res<VoxelWorld>,
    mut query: Query<(&mut Transform, &mut Velocity), (With<Mob>, Without<Player>)>,
) {
    for (mut transform, mut velocity) in query.iter_mut() {
        velocity.0.y += GRAVITY * time.delta_secs();

        let new_pos = transform.translation + velocity.0 * time.delta_secs();

        // Simple collision for mobs
        let mob_aabb = PlayerAABB {
            half_width: 0.4,
            half_height: 0.4,
        };

        if !check_collision(
            &voxel_world,
            Vec3::new(new_pos.x, transform.translation.y, transform.translation.z),
            &mob_aabb,
        ) {
            transform.translation.x = new_pos.x;
        }
        if !check_collision(
            &voxel_world,
            Vec3::new(transform.translation.x, transform.translation.y, new_pos.z),
            &mob_aabb,
        ) {
            transform.translation.z = new_pos.z;
        }
        if !check_collision(
            &voxel_world,
            Vec3::new(transform.translation.x, new_pos.y, transform.translation.z),
            &mob_aabb,
        ) {
            transform.translation.y = new_pos.y;
        } else {
            if velocity.0.y < 0.0 {
                let feet_y = new_pos.y - 0.4;
                let block_y = feet_y.floor() + 1.0;
                transform.translation.y = block_y + 0.4;
            }
            velocity.0.y = 0.0;
        }
    }
}

fn zombie_attack_player(
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    zombie_query: Query<(&Transform, &MobAI), (With<Mob>, With<MobType>)>,
) {
    let Ok((player_transform, mut player_health)) = player_query.get_single_mut() else {
        return;
    };

    for (zombie_transform, ai) in zombie_query.iter() {
        if ai.state == AIState::Attacking {
            let dist = zombie_transform
                .translation
                .distance(player_transform.translation);
            if dist < ZOMBIE_ATTACK_RANGE {
                player_health.0 =
                    (player_health.0 - ZOMBIE_ATTACK_DAMAGE * time.delta_secs()).max(0.0);
            }
        }
    }
}

// ============================================================================
// COMBAT & DROPS
// ============================================================================

fn player_attack(
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mob_query: Query<(Entity, &Transform), With<Mob>>,
    mut mob_hit_events: EventWriter<MobHit>,
    game_ui: Res<GameUI>,
) {
    if game_ui.inventory_open || game_ui.crafting_open {
        return;
    }
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(camera) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera.translation();
    let ray_dir = camera.forward().as_vec3();

    // Check for mob hits (simple sphere check)
    for (entity, transform) in mob_query.iter() {
        let to_mob = transform.translation - ray_origin;
        let t = to_mob.dot(ray_dir);
        if t < 0.0 || t > 5.0 {
            continue;
        }

        let closest = ray_origin + ray_dir * t;
        if closest.distance(transform.translation) < 1.0 {
            mob_hit_events.send(MobHit {
                entity,
                damage: PLAYER_ATTACK_DAMAGE,
            });
            break;
        }
    }
}

fn process_mob_damage(
    mut commands: Commands,
    mut events: EventReader<MobHit>,
    mut mob_query: Query<
        (
            &mut Health,
            &Transform,
            &MobType,
            &mut Velocity,
            Option<&HitFlash>,
        ),
        With<Mob>,
    >,
    player_query: Query<&Transform, With<Player>>,
    item_assets: Res<ItemDropAssets>,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    for event in events.read() {
        let Ok((mut health, transform, mob_type, mut velocity, has_flash)) =
            mob_query.get_mut(event.entity)
        else {
            continue;
        };

        health.0 -= event.damage;

        // Add knockback
        let knockback_dir = (transform.translation - player_pos).normalize_or_zero();
        velocity.0 += knockback_dir * 5.0 + Vec3::Y * 3.0;

        // Add hit flash effect (red flash) if not already flashing
        if has_flash.is_none() {
            // Get the mob's base color for later restoration
            let original_color = match mob_type {
                MobType::Pig => Color::srgb(0.95, 0.75, 0.7),
                MobType::Sheep => Color::srgb(0.95, 0.95, 0.95),
                MobType::Zombie => Color::srgb(0.4, 0.6, 0.4),
            };
            commands.entity(event.entity).insert(HitFlash {
                timer: 0.15,
                original_color,
            });
        }

        if health.0 <= 0.0 {
            commands.entity(event.entity).despawn_recursive();

            // Spawn drops
            let (item_type, count) = match mob_type {
                MobType::Pig => (ItemType::RawPork, 1 + (fastrand::u32(..) % 3)),
                MobType::Sheep => (ItemType::Wool, 1 + (fastrand::u32(..) % 2)),
                MobType::Zombie => (ItemType::RottenFlesh, fastrand::u32(..) % 3),
            };

            if count > 0 {
                commands.spawn((
                    DroppedItem { item_type, count },
                    Mesh3d(item_assets.mesh.clone()),
                    MeshMaterial3d(item_assets.material.clone()),
                    Transform::from_translation(transform.translation + Vec3::Y * 0.5),
                    ItemBob {
                        base_y: transform.translation.y + 0.5,
                        time: 0.0,
                    },
                ));
            }
        }
    }
}

fn item_pickup(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    item_query: Query<(Entity, &Transform, &DroppedItem)>,
    mut inventory: ResMut<Inventory>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (entity, item_transform, dropped_item) in item_query.iter() {
        if player_transform
            .translation
            .distance(item_transform.translation)
            < ITEM_PICKUP_RANGE
        {
            if inventory.add_item(dropped_item.item_type, dropped_item.count) {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn item_bob(time: Res<Time>, mut query: Query<(&mut Transform, &mut ItemBob)>) {
    for (mut transform, mut bob) in query.iter_mut() {
        bob.time += time.delta_secs();
        transform.translation.y = bob.base_y + (bob.time * 2.0).sin() * 0.1;
        transform.rotate_y(time.delta_secs());
    }
}

fn animate_mobs(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut MobAnimation, &MobAI), With<Mob>>,
) {
    for (mut transform, mut anim, ai) in query.iter_mut() {
        anim.time += time.delta_secs();
        anim.is_moving = ai.state == AIState::Wandering || ai.state == AIState::Chasing;

        // Gentle bobbing animation for all mobs
        let bob_speed = if anim.is_moving { 8.0 } else { 2.0 };
        let bob_amount = if anim.is_moving { 0.05 } else { 0.02 };
        let bob_offset = (anim.time * bob_speed).sin() * bob_amount;

        // Apply a small vertical offset (relative to base position)
        // We only modify Y slightly for breathing/bobbing effect
        let base_y = transform.translation.y;
        transform.translation.y = base_y + bob_offset * time.delta_secs() * 10.0;

        // Slight rotation wobble when moving
        if anim.is_moving {
            let wobble = (anim.time * 4.0).sin() * 0.02;
            transform.rotate_z(wobble * time.delta_secs());
        }
    }
}

// ============================================================================
// BLOCK INTERACTION
// ============================================================================

fn block_raycast(
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    voxel_world: Res<VoxelWorld>,
    mut raycast_events: EventWriter<RaycastHit>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    if let Some((coord, normal)) = dda_raycast(ray_origin, ray_direction, &voxel_world, 100) {
        raycast_events.send(RaycastHit { coord, normal });
    }
}

fn dda_raycast(
    origin: Vec3,
    direction: Vec3,
    voxel_world: &VoxelWorld,
    max_steps: i32,
) -> Option<(IVec3, IVec3)> {
    let mut current = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    let step = IVec3::new(
        if direction.x >= 0.0 { 1 } else { -1 },
        if direction.y >= 0.0 { 1 } else { -1 },
        if direction.z >= 0.0 { 1 } else { -1 },
    );

    let t_delta = Vec3::new(
        if direction.x.abs() < 1e-10 {
            f32::MAX
        } else {
            (1.0 / direction.x).abs()
        },
        if direction.y.abs() < 1e-10 {
            f32::MAX
        } else {
            (1.0 / direction.y).abs()
        },
        if direction.z.abs() < 1e-10 {
            f32::MAX
        } else {
            (1.0 / direction.z).abs()
        },
    );

    let mut t_max = Vec3::new(
        if direction.x >= 0.0 {
            ((current.x + 1) as f32 - origin.x) * t_delta.x
        } else {
            (origin.x - current.x as f32) * t_delta.x
        },
        if direction.y >= 0.0 {
            ((current.y + 1) as f32 - origin.y) * t_delta.y
        } else {
            (origin.y - current.y as f32) * t_delta.y
        },
        if direction.z >= 0.0 {
            ((current.z + 1) as f32 - origin.z) * t_delta.z
        } else {
            (origin.z - current.z as f32) * t_delta.z
        },
    );

    let mut last_normal = IVec3::ZERO;

    for _ in 0..max_steps {
        if voxel_world.blocks.contains_key(&current) {
            return Some((current, last_normal));
        }

        if t_max.x < t_max.y && t_max.x < t_max.z {
            current.x += step.x;
            t_max.x += t_delta.x;
            last_normal = IVec3::new(-step.x, 0, 0);
        } else if t_max.y < t_max.z {
            current.y += step.y;
            t_max.y += t_delta.y;
            last_normal = IVec3::new(0, -step.y, 0);
        } else {
            current.z += step.z;
            t_max.z += t_delta.z;
            last_normal = IVec3::new(0, 0, -step.z);
        }
    }

    None
}

fn block_modification(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut raycast_events: EventReader<RaycastHit>,
    mut voxel_world: ResMut<VoxelWorld>,
    cube_mesh: Res<CubeMesh>,
    material_handles: Res<MaterialHandles>,
    mut inventory: ResMut<Inventory>,
    game_ui: Res<GameUI>,
) {
    if game_ui.inventory_open || game_ui.crafting_open {
        return;
    }

    let Some(hit) = raycast_events.read().last() else {
        return;
    };

    // Left click: break block (if not hitting a mob)
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some((block_type, entity)) = voxel_world.blocks.remove(&hit.coord) {
            commands.entity(entity).despawn();
            inventory.add_item(ItemType::Block(block_type), 1);
        }
    }

    // Right click: place block from inventory
    if mouse_button.just_pressed(MouseButton::Right) {
        let new_coord = hit.coord + hit.normal;

        if voxel_world.blocks.contains_key(&new_coord) {
            return;
        }

        // Check if selected slot has a block
        if let Some(stack) = &inventory.slots[inventory.selected_slot] {
            if let ItemType::Block(block_type) = stack.item_type {
                let material = material_handles.materials[block_type as usize].clone();

                let entity = commands
                    .spawn((
                        Mesh3d(cube_mesh.0.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(new_coord.as_vec3()),
                        block_type,
                        Block,
                    ))
                    .id();

                voxel_world.blocks.insert(new_coord, (block_type, entity));
                inventory.remove_selected();
            }
        }
    }
}

// ============================================================================
// UI SYSTEMS
// ============================================================================

fn update_survival_ui(
    player_query: Query<(&Health, &Hunger, &Stamina), With<Player>>,
    mut health_bar: Query<&mut Node, (With<HealthBar>, Without<HungerBar>, Without<StaminaBar>)>,
    mut hunger_bar: Query<&mut Node, (With<HungerBar>, Without<HealthBar>, Without<StaminaBar>)>,
    mut stamina_bar: Query<&mut Node, (With<StaminaBar>, Without<HealthBar>, Without<HungerBar>)>,
) {
    let Ok((health, hunger, stamina)) = player_query.get_single() else {
        return;
    };

    if let Ok(mut node) = health_bar.get_single_mut() {
        node.width = Val::Percent(health.0);
    }
    if let Ok(mut node) = hunger_bar.get_single_mut() {
        node.width = Val::Percent(hunger.0);
    }
    if let Ok(mut node) = stamina_bar.get_single_mut() {
        node.width = Val::Percent(stamina.0);
    }
}

fn update_hotbar_ui(
    inventory: Res<Inventory>,
    mut hotbar_slots: Query<(&HotbarSlot, &Children, &mut BorderColor)>,
    mut icon_query: Query<(&HotbarItemIcon, &mut BackgroundColor), Without<HotbarSlot>>,
    mut text_query: Query<&mut Text, Without<SelectedItemName>>,
    mut item_name_query: Query<&mut Text, With<SelectedItemName>>,
) {
    // Update hotbar slot contents
    for (slot, children, mut border) in hotbar_slots.iter_mut() {
        // Update border color for selection
        border.0 = if slot.0 == inventory.selected_slot {
            Color::WHITE
        } else {
            Color::srgba(0.4, 0.4, 0.4, 0.8)
        };

        if let Some(stack) = &inventory.slots[slot.0] {
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = if stack.count > 1 {
                        format!("{}", stack.count)
                    } else {
                        String::new()
                    };
                }
            }
        } else {
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = String::new();
                }
            }
        }
    }

    // Update hotbar item icons (colored squares)
    for (icon, mut bg) in icon_query.iter_mut() {
        if let Some(stack) = &inventory.slots[icon.0] {
            bg.0 = stack.item_type.color();
        } else {
            bg.0 = Color::NONE;
        }
    }

    // Update selected item name
    if let Ok(mut name_text) = item_name_query.get_single_mut() {
        if let Some(stack) = &inventory.slots[inventory.selected_slot] {
            name_text.0 = stack.item_type.display_name().to_string();
        } else {
            name_text.0 = String::new();
        }
    }
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut fps_text: Query<&mut Text, With<FpsText>>) {
    use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

    if let Ok(mut text) = fps_text.get_single_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.0 = format!("FPS: {:.0}", value);
            }
        }
    }
}

// ============================================================================
// DAY/NIGHT CYCLE SYSTEM
// ============================================================================

fn update_day_night_cycle(
    time: Res<Time>,
    mut cycle: ResMut<DayNightCycle>,
    mut sun_query: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut ambient: ResMut<AmbientLight>,
    mut clear_color: ResMut<ClearColor>,
    mut fog_query: Query<&mut DistanceFog>,
) {
    // Advance time
    cycle.time += time.delta_secs() / cycle.day_length_seconds;
    if cycle.time > 1.0 {
        cycle.time -= 1.0;
    }

    // Update sun position and intensity
    if let Ok((mut light, mut transform)) = sun_query.get_single_mut() {
        // Sun rotates around the world
        let angle = cycle.time * PI * 2.0;
        let sun_distance = 100.0;
        transform.translation =
            Vec3::new(angle.cos() * sun_distance, angle.sin() * sun_distance, 0.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);

        // Adjust sun intensity
        light.illuminance = cycle.sun_intensity() * 20000.0;
    }

    // Update sky color
    clear_color.0 = cycle.sky_color();

    // Update ambient light
    ambient.color = cycle.ambient_color();
    ambient.brightness = if cycle.time > 0.25 && cycle.time < 0.75 {
        500.0
    } else {
        100.0
    };

    // Update fog color to match sky
    for mut fog in fog_query.iter_mut() {
        fog.color = cycle.sky_color();
    }
}

// ============================================================================
// HIT FEEDBACK SYSTEM
// ============================================================================

fn hit_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitFlash, &Children)>,
    mut material_query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut flash, children) in query.iter_mut() {
        flash.timer -= time.delta_secs();

        if flash.timer <= 0.0 {
            // Restore original colors
            for &child in children.iter() {
                if let Ok(mat_handle) = material_query.get_mut(child) {
                    if let Some(mat) = materials.get_mut(mat_handle.0.id()) {
                        mat.base_color = flash.original_color;
                    }
                }
            }
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

fn spawn_damage_number(commands: &mut Commands, position: Vec3, damage: f32) {
    // Damage numbers are spawned as Text2d entities in the UI layer
    // For simplicity, we'll skip this for now as it requires more complex setup
    let _ = (commands, position, damage);
}

// ============================================================================
// APP ENTRY POINT
// ============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Survival".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // Resources
        .init_resource::<VoxelWorld>()
        .init_resource::<Inventory>()
        .init_resource::<CraftingGrid>()
        .init_resource::<CraftingRecipes>()
        .init_resource::<GameUI>()
        // Events
        .add_event::<RaycastHit>()
        .add_event::<HungerDepleted>()
        .add_event::<MobHit>()
        // Startup
        .add_systems(
            Startup,
            (
                init_assets,
                setup_world.after(init_assets),
                spawn_player.after(setup_world),
                spawn_mobs.after(init_assets),
                setup_ui.after(spawn_player),
                grab_cursor.after(setup_ui),
            ),
        )
        // FixedUpdate (physics)
        .add_systems(
            FixedUpdate,
            (hunger_decay, starvation_damage, apply_physics, mob_physics).chain(),
        )
        // Update
        .add_systems(
            Update,
            (
                player_look,
                player_movement,
                hotbar_selection,
                toggle_menus,
                handle_pause_buttons,
                mob_ai,
                zombie_attack_player,
                player_attack,
                process_mob_damage,
                item_pickup,
                item_bob,
                animate_mobs,
                block_raycast,
                block_modification.after(block_raycast),
                hit_flash_system,
            ),
        )
        // PostUpdate
        .add_systems(
            PostUpdate,
            (update_survival_ui, update_hotbar_ui, update_fps),
        )
        .run();
}

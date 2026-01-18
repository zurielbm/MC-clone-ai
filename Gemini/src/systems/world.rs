use crate::components::{BlockMarker, BlockType, MainCamera};
use crate::resources::{CubeMesh, MaterialHandles, RaycastHit, VoxelWorld};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowCaster};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use rand::Rng;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SelectionMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material for SelectionMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/selection.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

pub fn init_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut selection_materials: ResMut<Assets<SelectionMaterial>>,
) {
    let mesh_handle = meshes.add(Cuboid::from_size(Vec3::ONE));
    commands.insert_resource(CubeMesh(mesh_handle));

    let material_handles = MaterialHandles {
        grass: standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.48, 0.32),
            unlit: true,
            ..default()
        }),
        dirt: standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.26, 0.18),
            unlit: true,
            ..default()
        }),
        stone: standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.42, 0.45, 0.48),
            unlit: true,
            ..default()
        }),
        wood: standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.32, 0.18, 0.12),
            unlit: true,
            ..default()
        }),
        leaves: standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.42, 0.22),
            unlit: true,
            ..default()
        }),
    };
    commands.insert_resource(material_handles);

    // Selection Box Asset (Shader Based)
    let selection_mesh = meshes.add(Cuboid::from_size(Vec3::splat(1.02)));
    let selection_mat = selection_materials.add(SelectionMaterial {
        color: LinearRgba::from(Color::srgb(0.0, 1.0, 0.5)),
    });
    commands.spawn((
        Mesh3d(selection_mesh),
        MeshMaterial3d(selection_mat),
        SelectionBox,
        Visibility::Hidden,
    ));
}

#[derive(Component)]
pub struct SelectionBox;

pub fn setup_world(
    mut commands: Commands,
    cube_mesh: Res<CubeMesh>,
    materials: Res<MaterialHandles>,
) {
    let mut world = VoxelWorld::default();

    for x in -16..16 {
        for z in -16..16 {
            for y in 0..4 {
                let coord = IVec3::new(x, y, z);
                let block_type = if y == 3 {
                    BlockType::Grass
                } else if y > 1 {
                    BlockType::Dirt
                } else {
                    BlockType::Stone
                };

                world.blocks.insert(coord, block_type);
            }
        }
    }

    // Now spawn entities only for surface blocks (occlusion culling)
    let block_coords: Vec<IVec3> = world.blocks.keys().cloned().collect();
    for coord in block_coords {
        let mut is_exposed = false;
        let neighbors = [
            IVec3::new(1, 0, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, -1, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 0, -1),
        ];

        for offset in neighbors {
            if !world.blocks.contains_key(&(coord + offset)) {
                is_exposed = true;
                break;
            }
        }

        if is_exposed {
            let block_type = world.blocks[&coord];
            let material = match block_type {
                BlockType::Grass => materials.grass.clone(),
                BlockType::Dirt => materials.dirt.clone(),
                BlockType::Stone => materials.stone.clone(),
                BlockType::Wood => materials.wood.clone(),
                BlockType::Leaves => materials.leaves.clone(),
            };

            let entity = commands
                .spawn((
                    Mesh3d(cube_mesh.0.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(coord.as_vec3()),
                    block_type,
                    BlockMarker(coord),
                    NotShadowCaster,
                ))
                .id();
            world.entities.insert(coord, entity);
        }
    }

    // Random Trees
    let mut rng = rand::rng();
    for _ in 0..20 {
        let x = rng.random_range(-14..14);
        let z = rng.random_range(-14..14);
        let coord = IVec3::new(x, 4, z); // Start above top layer
        spawn_tree(coord, &mut commands, &cube_mesh.0, &materials, &mut world);
    }

    commands.insert_resource(world);

    // Sun/Moon Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun,
    ));
}

#[derive(Component)]
pub struct Sun;

pub fn day_night_cycle(
    mut time_of_day: ResMut<crate::resources::TimeOfDay>,
    time: Res<Time>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut camera_query: Query<&mut Camera, With<crate::components::MainCamera>>,
) {
    // Only update once every few frames or keep it simple
    let day_duration = 60.0;
    time_of_day.0 += time.delta_secs() / day_duration;
    if time_of_day.0 > 1.0 {
        time_of_day.0 -= 1.0;
    }

    let angle = time_of_day.0 * std::f32::consts::TAU;
    let sun_y = angle.sin();

    if let Ok((mut transform, mut light)) = sun_query.get_single_mut() {
        let rot = Quat::from_rotation_x(angle);
        transform.translation = rot * Vec3::new(0.0, 20.0, 0.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);

        // Dynamic light intensity
        light.illuminance = (sun_y.max(0.0) * 10000.0).max(500.0);
    }

    if let Ok(mut camera) = camera_query.get_single_mut() {
        let sky_color = if sun_y < -0.1 {
            Color::srgb(0.02, 0.02, 0.05) // Dark Night
        } else if sun_y < 0.2 {
            Color::srgb(0.8, 0.4, 0.2) // Sunset/Sunrise
        } else {
            Color::srgb(0.5, 0.7, 1.0) // Day
        };
        camera.clear_color = ClearColorConfig::Custom(sky_color);
    }
}

pub fn update_targeting(
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    world: Res<VoxelWorld>,
    mut selection_query: Query<(&mut Transform, &mut Visibility), With<SelectionBox>>,
) {
    let Ok((cam_transform, _)) = camera_query.get_single() else {
        return;
    };
    let ray_origin = cam_transform.translation();
    let ray_dir = cam_transform.forward();

    // Simple DDA Raycast for targeting (copy logic from block_raycast but run every frame)
    let mut map_pos = IVec3::new(
        ray_origin.x.floor() as i32,
        ray_origin.y.floor() as i32,
        ray_origin.z.floor() as i32,
    );
    let delta_dist = Vec3::new(
        (1.0 / ray_dir.x).abs(),
        (1.0 / ray_dir.y).abs(),
        (1.0 / ray_dir.z).abs(),
    );
    let step = IVec3::new(
        if ray_dir.x < 0.0 { -1 } else { 1 },
        if ray_dir.y < 0.0 { -1 } else { 1 },
        if ray_dir.z < 0.0 { -1 } else { 1 },
    );
    let mut side_dist = Vec3::new(
        if ray_dir.x < 0.0 {
            (ray_origin.x - map_pos.x as f32) * delta_dist.x
        } else {
            (map_pos.x as f32 + 1.0 - ray_origin.x) * delta_dist.x
        },
        if ray_dir.y < 0.0 {
            (ray_origin.y - map_pos.y as f32) * delta_dist.y
        } else {
            (map_pos.y as f32 + 1.0 - ray_origin.y) * delta_dist.y
        },
        if ray_dir.z < 0.0 {
            (ray_origin.z - map_pos.z as f32) * delta_dist.z
        } else {
            (map_pos.z as f32 + 1.0 - ray_origin.z) * delta_dist.z
        },
    );

    let max_dist = 6.0; // Reach distance
    let mut dist = 0.0;
    let mut hit = false;

    while dist < max_dist {
        if world.blocks.contains_key(&map_pos) {
            hit = true;
            break;
        }
        if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
            dist = side_dist.x;
            side_dist.x += delta_dist.x;
            map_pos.x += step.x;
        } else if side_dist.y < side_dist.z {
            dist = side_dist.y;
            side_dist.y += delta_dist.y;
            map_pos.y += step.y;
        } else {
            dist = side_dist.z;
            side_dist.z += delta_dist.z;
            map_pos.z += step.z;
        }
    }

    if let Ok((mut selection_transform, mut visibility)) = selection_query.get_single_mut() {
        if hit {
            *visibility = Visibility::Visible;
            selection_transform.translation = map_pos.as_vec3();
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn spawn_tree(
    coord: IVec3,
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    materials: &MaterialHandles,
    world: &mut crate::resources::VoxelWorld,
) {
    // Trunk
    for i in 0..4 {
        let p = coord + IVec3::new(0, i, 0);
        if !world.blocks.contains_key(&p) {
            let entity = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(materials.wood.clone()),
                    Transform::from_translation(p.as_vec3()),
                    BlockType::Wood,
                    BlockMarker(p),
                    NotShadowCaster,
                ))
                .id();
            world.blocks.insert(p, BlockType::Wood);
            world.entities.insert(p, entity);
        }
    }

    // Leaves
    let leaf_center = coord + IVec3::new(0, 4, 0);
    for x in -2..=2 {
        for y in -1..=1 {
            for z in -2..=2 {
                let p = leaf_center + IVec3::new(x, y, z);
                // Simple sphere/box leaves
                if x.abs() + y.abs() + z.abs() <= 3 && !world.blocks.contains_key(&p) {
                    let entity = commands
                        .spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(materials.leaves.clone()),
                            Transform::from_translation(p.as_vec3()),
                            BlockType::Leaves,
                            BlockMarker(p),
                            NotShadowCaster,
                        ))
                        .id();
                    world.blocks.insert(p, BlockType::Leaves);
                    world.entities.insert(p, entity);
                }
            }
        }
    }
}

pub fn block_raycast(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    world: Res<VoxelWorld>,
    mut raycast_events: EventWriter<RaycastHit>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) && !mouse_input.just_pressed(MouseButton::Right)
    {
        return;
    }

    let Ok((transform, _)) = camera_query.get_single() else {
        return;
    };
    let ray_origin = transform.translation();
    let ray_dir = transform.forward();

    // DDA Algorithm
    let mut map_pos = IVec3::new(
        ray_origin.x.floor() as i32,
        ray_origin.y.floor() as i32,
        ray_origin.z.floor() as i32,
    );

    let delta_dist = Vec3::new(
        (1.0 / ray_dir.x).abs(),
        (1.0 / ray_dir.y).abs(),
        (1.0 / ray_dir.z).abs(),
    );

    let step = IVec3::new(
        if ray_dir.x < 0.0 { -1 } else { 1 },
        if ray_dir.y < 0.0 { -1 } else { 1 },
        if ray_dir.z < 0.0 { -1 } else { 1 },
    );

    let mut side_dist = Vec3::new(
        if ray_dir.x < 0.0 {
            (ray_origin.x - map_pos.x as f32) * delta_dist.x
        } else {
            (map_pos.x as f32 + 1.0 - ray_origin.x) * delta_dist.x
        },
        if ray_dir.y < 0.0 {
            (ray_origin.y - map_pos.y as f32) * delta_dist.y
        } else {
            (map_pos.y as f32 + 1.0 - ray_origin.y) * delta_dist.y
        },
        if ray_dir.z < 0.0 {
            (ray_origin.z - map_pos.z as f32) * delta_dist.z
        } else {
            (map_pos.z as f32 + 1.0 - ray_origin.z) * delta_dist.z
        },
    );

    let mut last_normal = IVec3::ZERO;
    let max_dist = 10.0;
    let mut dist = 0.0;

    while dist < max_dist {
        if world.blocks.contains_key(&map_pos) {
            let hit_entity = world.entities.get(&map_pos).cloned();
            raycast_events.send(RaycastHit {
                coord: map_pos,
                normal: last_normal,
                entity: hit_entity,
            });
            return;
        }

        if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
            dist = side_dist.x;
            side_dist.x += delta_dist.x;
            map_pos.x += step.x;
            last_normal = IVec3::new(-step.x, 0, 0);
        } else if side_dist.y < side_dist.z {
            dist = side_dist.y;
            side_dist.y += delta_dist.y;
            map_pos.y += step.y;
            last_normal = IVec3::new(0, -step.y, 0);
        } else {
            dist = side_dist.z;
            side_dist.z += delta_dist.z;
            map_pos.z += step.z;
            last_normal = IVec3::new(0, 0, -step.z);
        }
    }
}

pub fn block_modification(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut raycast_events: EventReader<RaycastHit>,
    mut world: ResMut<VoxelWorld>,
    mut inventory: ResMut<crate::resources::Inventory>,
    cube_mesh: Res<CubeMesh>,
    materials: Res<MaterialHandles>,
) {
    for event in raycast_events.read() {
        if mouse_input.just_pressed(MouseButton::Left) {
            // Remove block
            if let Some(block_type) = world.blocks.remove(&event.coord) {
                if let Some(entity) = world.entities.remove(&event.coord) {
                    commands.entity(entity).despawn_recursive();
                }
                // Add to inventory
                *inventory.items.entry(block_type).or_insert(0) += 1;

                // Reveal neighbors
                let neighbors = [
                    IVec3::new(1, 0, 0),
                    IVec3::new(-1, 0, 0),
                    IVec3::new(0, 1, 0),
                    IVec3::new(0, -1, 0),
                    IVec3::new(0, 0, 1),
                    IVec3::new(0, 0, -1),
                ];

                for offset in neighbors {
                    let neighbor_coord = event.coord + offset;
                    if let Some(&neighbor_type) = world.blocks.get(&neighbor_coord) {
                        if !world.entities.contains_key(&neighbor_coord) {
                            let material = match neighbor_type {
                                BlockType::Grass => materials.grass.clone(),
                                BlockType::Dirt => materials.dirt.clone(),
                                BlockType::Stone => materials.stone.clone(),
                                BlockType::Wood => materials.wood.clone(),
                                BlockType::Leaves => materials.leaves.clone(),
                            };

                            let entity = commands
                                .spawn((
                                    Mesh3d(cube_mesh.0.clone()),
                                    MeshMaterial3d(material),
                                    Transform::from_translation(neighbor_coord.as_vec3()),
                                    neighbor_type,
                                    BlockMarker(neighbor_coord),
                                    NotShadowCaster,
                                ))
                                .id();
                            world.entities.insert(neighbor_coord, entity);
                        }
                    }
                }
            }
        } else if mouse_input.just_pressed(MouseButton::Right) {
            // Add block from inventory
            // Just pick the first available block for now
            let available_block = inventory
                .items
                .iter()
                .filter(|entry| *entry.1 > 0)
                .map(|entry| *entry.0)
                .next();

            if let Some(block_type) = available_block {
                let new_pos = event.coord + event.normal;
                if !world.blocks.contains_key(&new_pos) {
                    let material = match block_type {
                        BlockType::Grass => materials.grass.clone(),
                        BlockType::Dirt => materials.dirt.clone(),
                        BlockType::Stone => materials.stone.clone(),
                        BlockType::Wood => materials.wood.clone(),
                        BlockType::Leaves => materials.leaves.clone(),
                    };

                    let entity = commands
                        .spawn((
                            Mesh3d(cube_mesh.0.clone()),
                            MeshMaterial3d(material),
                            Transform::from_translation(new_pos.as_vec3()),
                            block_type,
                            BlockMarker(new_pos),
                            NotShadowCaster,
                        ))
                        .id();

                    world.blocks.insert(new_pos, block_type);
                    world.entities.insert(new_pos, entity);

                    // Consume from inventory
                    if let Some(count) = inventory.items.get_mut(&block_type) {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                }
            }
        }
    }
}

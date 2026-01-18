use crate::components::{Grounded, Velocity};
use crate::resources::VoxelWorld;
use bevy::prelude::*;

pub fn apply_physics(
    mut query: Query<(&mut Transform, &mut Velocity, &mut Grounded)>,
    world: Res<VoxelWorld>,
    time: Res<Time<Fixed>>,
) {
    let delta = time.delta_secs();
    let gravity = -9.81;
    let player_radius = 0.3;
    let player_height = 1.8;
    let half_height = player_height / 2.0;

    for (mut transform, mut velocity, mut grounded) in query.iter_mut() {
        // Apply gravity
        velocity.y += gravity * delta;

        // Current position
        let mut pos = transform.translation;

        // Try Y movement first
        let mut next_y = pos.y + velocity.y * delta;
        let mut hit_y = false;
        let check_y = if velocity.y < 0.0 {
            next_y - half_height
        } else {
            next_y + half_height
        };

        // Check corners and center
        let horizontal_offsets = [
            Vec3::new(player_radius, 0.0, player_radius),
            Vec3::new(player_radius, 0.0, -player_radius),
            Vec3::new(-player_radius, 0.0, player_radius),
            Vec3::new(-player_radius, 0.0, -player_radius),
            Vec3::ZERO,
        ];

        for offset in horizontal_offsets.iter() {
            let p = Vec3::new(pos.x + offset.x, check_y, pos.z + offset.z);
            let block_pos = IVec3::new(p.x.round() as i32, p.y.round() as i32, p.z.round() as i32);
            if world.blocks.contains_key(&block_pos) {
                hit_y = true;
                break;
            }
        }

        if hit_y {
            if velocity.y < 0.0 {
                grounded.0 = true;
                let block_y = check_y.round() as f32;
                next_y = block_y + 0.5 + half_height;
            } else {
                let block_y = check_y.round() as f32;
                next_y = block_y - 0.5 - half_height;
            }
            velocity.y = 0.0;
        } else {
            grounded.0 = false;
        }
        pos.y = next_y;

        // Try X movement
        let mut next_x = pos.x + velocity.x * delta;
        let mut hit_x = false;
        let check_x = if velocity.x < 0.0 {
            next_x - player_radius
        } else {
            next_x + player_radius
        };

        // Check multiple heights (feet, waist, head)
        for h_off in [-half_height + 0.1, 0.0, half_height - 0.1] {
            let p = Vec3::new(check_x, pos.y + h_off, pos.z);
            let block_pos = IVec3::new(p.x.round() as i32, p.y.round() as i32, p.z.round() as i32);
            if world.blocks.contains_key(&block_pos) {
                hit_x = true;
                break;
            }
        }

        if hit_x {
            let block_x = check_x.round() as f32;
            next_x = block_x
                + (if velocity.x < 0.0 {
                    0.5 + player_radius
                } else {
                    -0.5 - player_radius
                });
            velocity.x = 0.0;
        }
        pos.x = next_x;

        // Try Z movement
        let mut next_z = pos.z + velocity.z * delta;
        let mut hit_z = false;
        let check_z = if velocity.z < 0.0 {
            next_z - player_radius
        } else {
            next_z + player_radius
        };

        for h_off in [-half_height + 0.1, 0.0, half_height - 0.1] {
            let p = Vec3::new(pos.x, pos.y + h_off, check_z);
            let block_pos = IVec3::new(p.x.round() as i32, p.y.round() as i32, p.z.round() as i32);
            if world.blocks.contains_key(&block_pos) {
                hit_z = true;
                break;
            }
        }

        if hit_z {
            let block_z = check_z.round() as f32;
            next_z = block_z
                + (if velocity.z < 0.0 {
                    0.5 + player_radius
                } else {
                    -0.5 - player_radius
                });
            velocity.z = 0.0;
        }
        pos.z = next_z;

        transform.translation = pos;
    }
}

pub fn ground_check() {
    // Merged into apply_physics
}

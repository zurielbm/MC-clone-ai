use crate::components::{Enemy, Grounded, Health, Mob, Passive, Player, Velocity};
use crate::resources::VoxelWorld;
use bevy::prelude::*;
use rand::Rng;

pub fn spawn_mobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::from_size(Vec3::splat(0.8)));
    let passive_mat = materials.add(Color::srgb(0.8, 0.8, 0.8));
    let enemy_mat = materials.add(Color::srgb(0.8, 0.2, 0.2));

    let mut rng = rand::rng();

    // Spawn within world bounds (-16..16)
    for _ in 0..6 {
        let x = rng.random_range(-14.0..14.0);
        let z = rng.random_range(-14.0..14.0);
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(passive_mat.clone()),
            Transform::from_xyz(x, 10.0, z),
            Mob,
            Passive,
            Velocity(Vec3::ZERO),
            Grounded(false),
            Health(20.0),
        ));
    }

    for _ in 0..4 {
        let x = rng.random_range(-14.0..14.0);
        let z = rng.random_range(-14.0..14.0);
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(enemy_mat.clone()),
            Transform::from_xyz(x, 10.0, z),
            Mob,
            Enemy,
            Velocity(Vec3::ZERO),
            Grounded(false),
            Health(20.0),
        ));
    }
}

pub fn mob_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut mob_query: Query<
        (
            &mut Transform,
            Option<&Passive>,
            Option<&Enemy>,
            &mut Velocity,
        ),
        (With<Mob>, Without<Player>),
    >,
) {
    let delta = time.delta_secs();
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    let mut rng = rand::rng();

    for (mut transform, passive, enemy, mut velocity) in mob_query.iter_mut() {
        if passive.is_some() {
            // Wander
            if rng.random_bool(0.01) {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                velocity.0.x = angle.cos() * 2.0;
                velocity.0.z = angle.sin() * 2.0;
            }
        } else if enemy.is_some() {
            // Chase
            let diff = player_pos - transform.translation;
            let dir = diff.normalize_or_zero();
            velocity.0.x = dir.x * 0.8;
            velocity.0.z = dir.z * 0.8;
        }

        // Apply horizontal movement
        transform.translation.x += velocity.0.x * delta;
        transform.translation.z += velocity.0.z * delta;
    }
}

pub fn mob_damage_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut player_health_query: Query<&mut Health, With<Player>>,
    mob_query: Query<&Transform, (With<Mob>, With<Enemy>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    if let Ok(mut health) = player_health_query.get_single_mut() {
        for mob_transform in mob_query.iter() {
            if mob_transform.translation.distance(player_pos) < 1.0 {
                health.0 -= 3.0 * time.delta_secs();
            }
        }
    }
}

pub fn mob_death(mut commands: Commands, query: Query<(Entity, &Health), With<Mob>>) {
    for (entity, health) in query.iter() {
        if health.0 <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn mob_attack(
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    mut mob_health_query: Query<
        (&Transform, &mut Health, &mut Velocity),
        (With<Mob>, Without<Player>),
    >,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(player_transform) = player_query.get_single() {
            let player_pos = player_transform.translation;
            let forward = player_transform.forward();

            for (mob_transform, mut health, mut velocity) in mob_health_query.iter_mut() {
                let to_mob = mob_transform.translation - player_pos;
                let dist = to_mob.length();
                let dot = forward.dot(to_mob.normalize_or_zero());

                // If mob is close and in front of player
                if dist < 3.5 && dot > 0.5 {
                    health.0 -= 10.0;

                    // Apply knockback
                    let knockback_force = 5.0;
                    let mut knockback_dir = to_mob.normalize_or_zero();
                    knockback_dir.y = 0.5; // Slight upward pop
                    velocity.0 += knockback_dir * knockback_force;
                }
            }
        }
    }
}

pub fn mob_boundary_check(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Velocity), With<Mob>>,
) {
    for (entity, mut transform, mut velocity) in query.iter_mut() {
        // Despawn if fell off
        if transform.translation.y < -10.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Clamp to world bounds
        let half_size = 15.5;
        if transform.translation.x.abs() > half_size {
            transform.translation.x = transform.translation.x.signum() * half_size;
            velocity.0.x = 0.0;
        }
        if transform.translation.z.abs() > half_size {
            transform.translation.z = transform.translation.z.signum() * half_size;
            velocity.0.z = 0.0;
        }
    }
}

pub fn update_mob_health_bars(
    mob_query: Query<(&Transform, &Health), With<Mob>>,
    mut gizmos: Gizmos,
) {
    for (transform, health) in mob_query.iter() {
        // Always show health bar
        let pos = transform.translation + Vec3::Y * 1.8;
        let width = (health.0 / 20.0).clamp(0.0, 1.0) * 0.8;

        // Background (Black)
        gizmos.line(pos - Vec3::X * 0.4, pos + Vec3::X * 0.4, Color::BLACK);
        // Health (Red)
        gizmos.line(
            pos - Vec3::X * 0.4,
            pos - Vec3::X * 0.4 + Vec3::X * width,
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}

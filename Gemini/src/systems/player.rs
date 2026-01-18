use crate::components::{Grounded, Health, Hunger, MainCamera, Player, Stamina, Velocity};
use crate::resources::GameState;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            Velocity::default(),
            Grounded(false),
            Health(100.0),
            Hunger(100.0),
            Stamina(100.0),
            Transform::from_xyz(0.0, 5.0, 0.0),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                MainCamera,
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.6, 0.0),
                Camera {
                    clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.15)),
                    hdr: true,
                    ..default()
                },
                Tonemapping::ReinhardLuminance,
                Bloom::default(),
                DistanceFog {
                    color: Color::srgb(0.1, 0.1, 0.15),
                    falloff: FogFalloff::Linear {
                        start: 10.0,
                        end: 40.0,
                    },
                    ..default()
                },
            ));
        });
}

pub fn player_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    if let (Ok(mut transform), Ok(mut camera_transform)) =
        (player_query.get_single_mut(), camera_query.get_single_mut())
    {
        let sensitivity = 0.002;

        for event in mouse_motion_events.read() {
            let delta = event.delta;

            // Player Y rotation (left/right)
            transform.rotate_y(-delta.x * sensitivity);

            // Camera X rotation (up/down)
            let mut new_rotation =
                camera_transform.rotation * Quat::from_rotation_x(-delta.y * sensitivity);

            // Clamp camera rotation to prevent flipping
            let (x, _, _) = new_rotation.to_euler(EulerRot::XYZ);
            if x < -1.5 {
                new_rotation = Quat::from_rotation_x(-1.5);
            } else if x > 1.5 {
                new_rotation = Quat::from_rotation_x(1.5);
            }

            camera_transform.rotation = new_rotation;
        }
    }
}

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut Velocity, &Grounded), With<Player>>,
    camera_query: Query<&Transform, (With<MainCamera>, Without<Player>)>,
    _time: Res<Time>,
) {
    if let (Ok((transform, mut velocity, grounded)), Ok(_camera_transform)) =
        (player_query.get_single_mut(), camera_query.get_single())
    {
        let speed = 5.0;
        let jump_force = 4.5;

        let mut direction = Vec3::ZERO;
        let forward = transform.forward();
        let right = transform.right();

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += *forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= *forward;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= *right;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *right;
        }

        direction.y = 0.0;
        if direction != Vec3::ZERO {
            direction = direction.normalize();
        }

        velocity.x = direction.x * speed;
        velocity.z = direction.z * speed;

        if keyboard_input.just_pressed(KeyCode::Space) && grounded.0 {
            velocity.y = jump_force;
        }
    }
}

pub fn grab_cursor(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    state: Res<State<GameState>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if let Ok(mut window) = windows.get_single_mut() {
        match state.get() {
            GameState::InGame => {
                // More aggressive locking: if it's NOT locked, lock it.
                // This helps when the window loses focus and regains it, or when the cursor "leaks"
                if window.cursor_options.grab_mode != CursorGrabMode::Locked {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                }
            }
            GameState::Paused | GameState::GameOver => {
                if window.cursor_options.grab_mode != CursorGrabMode::None {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                }
            }
        }
    }
}

pub fn pause_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::InGame => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::InGame),
            GameState::GameOver => {}
        }
    }

    if *state.get() == GameState::Paused && keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.send(AppExit::Success);
    }
}

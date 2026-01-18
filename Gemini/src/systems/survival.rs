use crate::components::{Health, Hunger, Player, Stamina};
use crate::resources::HungerDepleted;
use bevy::prelude::*;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HungerBar;

#[derive(Component)]
pub struct StaminaBar;

#[derive(Component)]
pub struct InventoryUI;

#[derive(Component)]
pub struct InventoryText;

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct FrameTimeBar;

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct DeathScreen;

#[derive(Component)]
pub struct RespawnButton;

pub fn hunger_decay(
    mut query: Query<&mut Hunger>,
    time: Res<Time>,
    mut events: EventWriter<HungerDepleted>,
) {
    for mut hunger in query.iter_mut() {
        hunger.0 -= 0.5 * time.delta_secs();
        if hunger.0 <= 0.0 {
            hunger.0 = 0.0;
            events.send(HungerDepleted);
        }
    }
}

pub fn starvation_damage(
    mut events: EventReader<HungerDepleted>,
    mut query: Query<&mut Health>,
    time: Res<Time>,
) {
    for _ in events.read() {
        for mut health in query.iter_mut() {
            health.0 -= 5.0 * time.delta_secs();
        }
    }
}

pub fn setup_ui(mut commands: Commands) {
    // Container
    commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::FlexStart,
            ..default()
        },))
        .with_children(|parent| {
            // Survival Stats Panel (Glassmorphism)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(15.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    BorderRadius::all(Val::Px(15.0)),
                ))
                .with_children(|stats_panel| {
                    // Health Bar
                    stats_panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn((
                                Text::new("HEALTH"),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                            p.spawn((
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(10.0),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderRadius::all(Val::Px(5.0)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.9, 0.1, 0.2)),
                                    HealthBar,
                                    BorderRadius::all(Val::Px(4.0)),
                                ));
                            });
                        });

                    // Hunger Bar
                    stats_panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn((
                                Text::new("HUNGER"),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                            p.spawn((
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(10.0),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderRadius::all(Val::Px(5.0)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.9, 0.5, 0.1)),
                                    HungerBar,
                                    BorderRadius::all(Val::Px(4.0)),
                                ));
                            });
                        });

                    // Stamina Bar
                    stats_panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn((
                                Text::new("STAMINA"),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                            p.spawn((
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(10.0),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                                BorderRadius::all(Val::Px(5.0)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.1, 0.6, 0.9)),
                                    StaminaBar,
                                    BorderRadius::all(Val::Px(4.0)),
                                ));
                            });
                        });
                }); // Close stats_panel

            // Diagnostics HUD (FPS & Performance Graph)
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.0),
                    top: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|diag| {
                    diag.spawn((
                        FpsText,
                        Text::new("FPS: 0"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    ));

                    // Simple Performance Graph (Frame Time Bar)
                    diag.spawn(Node {
                        width: Val::Px(200.0),
                        height: Val::Px(10.0),
                        margin: UiRect::top(Val::Px(5.0)),
                        ..default()
                    })
                    .with_children(|bar_container| {
                        bar_container.spawn((
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.0, 0.8, 1.0)),
                            FrameTimeBar,
                        ));
                    });
                });
        });

    // Simplified Crosshair (Minimal Dot)
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 1.0, 1.0, 0.8)),
                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                BorderRadius::all(Val::Px(2.0)),
            ));
        });
}

pub fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.9)),
                    BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("PAUSED"),
                        TextFont {
                            font_size: 60.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    p.spawn((
                        Text::new("Press ESC to Resume"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                    p.spawn((
                        Text::new("Press Q to Quit"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
        });
}

pub fn setup_death_screen(mut commands: Commands) {
    commands
        .spawn((
            DeathScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                display: Display::None,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("YOU DIED"),
                TextFont {
                    font_size: 100.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.1, 0.15)),
            ));

            parent
                .spawn((
                    RespawnButton,
                    Button,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(75.0),
                        margin: UiRect::top(Val::Px(50.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                    BorderColor(Color::WHITE),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("RESPAWN"),
                        TextFont {
                            font_size: 34.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn update_pause_menu_visibility(
    state: Res<State<crate::resources::GameState>>,
    mut query: Query<&mut Node, With<PauseMenu>>,
) {
    if let Ok(mut node) = query.get_single_mut() {
        node.display = match state.get() {
            crate::resources::GameState::Paused => Display::Flex,
            _ => Display::None,
        };
    }
}

pub fn update_death_screen(
    player_query: Query<&Health, With<Player>>,
    mut death_screen_query: Query<&mut Node, With<DeathScreen>>,
    mut next_state: ResMut<NextState<crate::resources::GameState>>,
    state: Res<State<crate::resources::GameState>>,
) {
    if let Ok(health) = player_query.get_single() {
        if health.0 <= 0.0 {
            if let Ok(mut node) = death_screen_query.get_single_mut() {
                node.display = Display::Flex;
            }
            if *state.get() != crate::resources::GameState::GameOver {
                next_state.set(crate::resources::GameState::GameOver);
            }
        } else {
            if let Ok(mut node) = death_screen_query.get_single_mut() {
                node.display = Display::None;
            }
        }
    }
}

pub fn respawn_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RespawnButton>),
    >,
    mut player_query: Query<(&mut Health, &mut Hunger, &mut Stamina, &mut Transform), With<Player>>,
    mut next_state: ResMut<NextState<crate::resources::GameState>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Ok((mut health, mut hunger, mut stamina, mut transform)) =
                    player_query.get_single_mut()
                {
                    health.0 = 100.0;
                    hunger.0 = 100.0;
                    stamina.0 = 100.0;
                    transform.translation = Vec3::new(0.0, 10.0, 0.0);
                    next_state.set(crate::resources::GameState::InGame);
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
        }
    }
}

pub fn update_survival_ui(
    player_query: Query<(&Health, &Hunger, &Stamina), With<Player>>,
    mut health_bar_query: Query<
        &mut Node,
        (With<HealthBar>, Without<HungerBar>, Without<StaminaBar>),
    >,
    mut hunger_bar_query: Query<
        &mut Node,
        (With<HungerBar>, Without<HealthBar>, Without<StaminaBar>),
    >,
    mut stamina_bar_query: Query<
        &mut Node,
        (With<StaminaBar>, Without<HealthBar>, Without<HungerBar>),
    >,
) {
    if let Ok((health, hunger, stamina)) = player_query.get_single() {
        if let Ok(mut node) = health_bar_query.get_single_mut() {
            node.width = Val::Percent(health.0.clamp(0.0, 100.0));
        }
        if let Ok(mut node) = hunger_bar_query.get_single_mut() {
            node.width = Val::Percent(hunger.0.clamp(0.0, 100.0));
        }
        if let Ok(mut node) = stamina_bar_query.get_single_mut() {
            node.width = Val::Percent(stamina.0.clamp(0.0, 100.0));
        }
    }
}

pub fn setup_inventory_ui(mut commands: Commands) {
    commands
        .spawn((
            InventoryUI,
            Node {
                width: Val::Px(250.0),
                height: Val::Auto,
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                display: Display::Flex,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderRadius::all(Val::Px(10.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("INVENTORY"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                InventoryText,
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            parent.spawn((
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new("CRAFTING"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
            ));

            parent.spawn((
                Text::new("[C] 4 Wood -> 1 Stone"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

pub fn update_inventory_ui(
    inventory: Res<crate::resources::Inventory>,
    mut query: Query<&mut Text, With<InventoryText>>,
) {
    if inventory.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            let mut content = String::new();
            for (item, count) in inventory.items.iter() {
                if *count > 0 {
                    content.push_str(&format!("{:?}: {}\n", item, count));
                }
            }
            if content.is_empty() {
                content = "Empty".to_string();
            }
            text.0 = content;
        }
    }
}

pub fn craft_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<crate::resources::Inventory>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let wood_count = inventory
            .items
            .get(&crate::components::BlockType::Wood)
            .cloned()
            .unwrap_or(0);
        if wood_count >= 4 {
            *inventory
                .items
                .get_mut(&crate::components::BlockType::Wood)
                .unwrap() -= 4;
            *inventory
                .items
                .entry(crate::components::BlockType::Stone)
                .or_insert(0) += 1;
        }
    }
}
pub fn update_diagnostics_ui(
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut fps_query: Query<&mut Text, With<FpsText>>,
    mut bar_query: Query<&mut Node, With<FrameTimeBar>>,
) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if let Ok(mut text) = fps_query.get_single_mut() {
                text.0 = format!("FPS: {:.0}", value);
            }
        }
    }

    if let Some(frame_time) =
        diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
    {
        if let Some(value) = frame_time.smoothed() {
            if let Ok(mut node) = bar_query.get_single_mut() {
                // Scale bar: 16.6ms (60fps) = 50% width
                let percentage = (value / 0.033) * 100.0; // 33ms is base
                node.width = Val::Percent(percentage.clamp(0.0, 100.0) as f32);
            }
        }
    }
}

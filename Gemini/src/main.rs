mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use resources::{GameState, HungerDepleted, RaycastHit};
use systems::mobs::{
    mob_ai, mob_attack, mob_boundary_check, mob_damage_player, mob_death, spawn_mobs,
    update_mob_health_bars,
};
use systems::physics::{apply_physics, ground_check};
use systems::player::{grab_cursor, pause_toggle, player_look, player_movement, spawn_player};
use systems::survival::{
    craft_system, hunger_decay, respawn_system, setup_death_screen, setup_inventory_ui,
    setup_pause_menu, setup_ui, starvation_damage, update_death_screen, update_diagnostics_ui,
    update_inventory_ui, update_pause_menu_visibility, update_survival_ui,
};
use systems::world::{
    SelectionMaterial, block_modification, block_raycast, day_night_cycle, init_assets,
    setup_world, update_targeting,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            MaterialPlugin::<SelectionMaterial>::default(),
        ))
        .init_state::<GameState>()
        .init_resource::<resources::TimeOfDay>()
        .init_resource::<resources::Inventory>()
        .add_event::<RaycastHit>()
        .add_event::<HungerDepleted>()
        .add_systems(
            Startup,
            (
                init_assets,
                setup_world,
                spawn_player,
                setup_ui,
                setup_pause_menu,
                setup_death_screen,
                spawn_mobs,
                setup_inventory_ui,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                pause_toggle,
                update_pause_menu_visibility,
                grab_cursor,
                update_death_screen,
                respawn_system.run_if(in_state(GameState::GameOver)),
            ),
        )
        .add_systems(
            Update,
            (
                player_look,
                player_movement,
                block_raycast,
                block_modification,
                update_targeting,
                update_survival_ui,
                day_night_cycle,
                mob_ai,
                mob_boundary_check,
                mob_attack,
                mob_damage_player,
                mob_death,
                update_mob_health_bars,
                update_inventory_ui,
                update_diagnostics_ui,
                craft_system,
            )
                .run_if(in_state(GameState::InGame))
                .chain(),
        )
        .add_systems(
            FixedUpdate,
            (hunger_decay, starvation_damage, apply_physics, ground_check)
                .run_if(in_state(GameState::InGame))
                .chain(),
        )
        .run();
}

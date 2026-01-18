Bevy Project Agent Guidelines

This document provides context and instructions for AI agents working on this Bevy project. Bevy is a data-driven game engine built in Rust using the Entity Component System (ECS) paradigm.

1. Project Overview

Engine: Bevy (Rust)

Architecture: Strict ECS (Entities, Components, Systems).

Paradigm: Data-oriented, modular, and parallel-first.

2. Core Technical Constraints

ECS Pattern

Components: Use simple Rust structs. They must implement the Component trait: #[derive(Component)].

Systems: Normal Rust functions. Prefer small, single-responsibility functions.

Entities: Treat as unique IDs. Do not store complex logic inside them.

Resources: Use for globally unique data (e.g., Time, AssetServer). Access via Res<T> or ResMut<T>.

App Structure

Every program is an App.

Logic is added via app.add_systems(Update, system_name) or app.add_systems(Startup, setup_system).

Use the Builder Pattern for App configuration.

Modularity & Plugins

All features should be grouped into Plugins.

Implement the Plugin trait for feature sets.

Use app.add_plugins(DefaultPlugins) for standard engine features (Renderer, UI, Input).

3. Coding Standards & Best Practices

System Management

Parallelism: Systems run in parallel by default. Do not assume execution order unless explicitly defined.

Ordering: Use .chain() when one system must run after another (e.g., update_player.before(move_player)).

Queries: Use Query<&Component> for read-only and Query<&mut Component> for mutable access.

Performance

Cargo.toml: Ensure optimization levels are set for dependencies in dev builds to avoid sluggish performance.

Dynamic Linking: Use bevy/dynamic_linking feature during development to speed up iterative compiles, but ensure it is removed for release.

Plugin Development (Third-Party)

Licensing: Default to dual-license (MIT / Apache 2.0).

Naming: Use bevy_ prefix only for relevant, non-generic plugin names.

Generics: Allow users to supply generic types to plugins for custom logic flexibility.

Crate Size: Keep dependencies minimal. Use default-features = false for the Bevy dependency.

4. Common Commands

Run: cargo run

Build: cargo build

Fast Iteration: cargo run --features bevy/dynamic_linking

Release Build: cargo build --release

5. Troubleshooting Context

GPU Errors: If the app panics with "Unable to find a GPU", verify Vulkan drivers are installed.

VSCode Debugging: On Windows, dynamic linking might require specific PATH adjustments in launch.json.

6. Development Workflow

Define data structures as Components or Resources.

Write Systems to process that data using Queries.

Bundle logic into a Plugin.

Register the Plugin in main.rs.
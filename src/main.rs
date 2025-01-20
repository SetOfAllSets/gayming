use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use blenvy::*;
use blueprints::spawn_from_blueprints::{
    BlueprintInfo, GameWorldTag, HideUntilReady, SpawnBlueprint,
};
use mesh_markers::MeshMarkerPlugin;
use player_controller::PlayerControllerPlugin;
mod player_controller;
mod mesh_markers;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, BlenvyPlugin::default()))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(PlayerControllerPlugin)
        .add_plugins(MeshMarkerPlugin)
        // We need to register components to make them visible to Blenvy
        .add_systems(Startup, setup)
        .run()
}

fn setup(world: &mut World) {
    world.commands().spawn((
        BlueprintInfo::from_path("levels/World.glb"),
        SpawnBlueprint,
        HideUntilReady,
        GameWorldTag,
    ));
}

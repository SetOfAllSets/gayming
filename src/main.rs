use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use blenvy::*;
use blueprints::spawn_from_blueprints::{BlueprintInfo, GameWorldTag, HideUntilReady, SpawnBlueprint};

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, BlenvyPlugin::default()))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        // We need to register components to make them visible to Blenvy
        .register_type::<Player>()
        .register_type::<ColliderConstructor>()
        .register_type::<GroundDetectionRay>()
        .add_systems(Startup, (setup, hooks))
        .add_systems(Update, move_player)
        .run()
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Player {
    speed: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GroundDetectionRay {}

fn setup(mut commands: Commands) {
    commands.spawn((
        BlueprintInfo::from_path("levels/World.glb"),
        SpawnBlueprint,
        HideUntilReady,
        GameWorldTag,
    ));
}

fn hooks(world: &mut World) {
    world.register_component_hooks::<GroundDetectionRay>().on_insert(|world, entity ,component_id| {
        println!("Component: {component_id:?} added to: {entity:?}");
    });
}

fn move_player(
    mut player: Query<(&mut Transform, &Player, &mut ExternalForce, &mut LinearVelocity)>,
    keys: Res<ButtonInput<KeyCode>>,
    timer: Res<Time>
) {
    for (mut transform, player, mut force, mut velocity) in &mut player {
        if keys.pressed(KeyCode::ArrowLeft) {
            transform.rotate_y(2.0*timer.delta_secs());
        }
        if keys.pressed(KeyCode::ArrowRight) {
            transform.rotate_y(-2.0*timer.delta_secs());
        }
        let mut movement = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            movement += transform.forward().as_vec3();
        }
        if keys.pressed(KeyCode::KeyS) {
            movement += transform.back().as_vec3();
        }
        if keys.pressed(KeyCode::KeyQ) {
            movement += transform.left().as_vec3();
        }
        if keys.pressed(KeyCode::KeyD) {
            movement += transform.right().as_vec3();
        }
        /*if keys.get_pressed().size_hint() == (0, Option::Some(0)) {
            friction.dynamic_coefficient = 2.0;
        } else {
            friction.dynamic_coefficient = 0.5;
        }*/
        movement = movement.normalize_or_zero() * player.speed;
        velocity.x = movement.x;
        velocity.z = movement.z;
    }
}

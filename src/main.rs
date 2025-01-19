#![feature(float_next_up_down)]

use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use blenvy::*;
use blueprints::spawn_from_blueprints::{
    BlueprintInfo, GameWorldTag, HideUntilReady, SpawnBlueprint,
};
mod player_controller;

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
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run()
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Player {
    speed: f32,
    air_speed: f32,
    jump_height: f32,
    was_moving: bool,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct JumpGraceTimer {
    timer: Timer,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GroundDetectionRay {}

fn setup(world: &mut World) {
    world.commands().spawn((
        BlueprintInfo::from_path("levels/World.glb"),
        SpawnBlueprint,
        HideUntilReady,
        GameWorldTag,
    ));
    world
        .register_component_hooks::<GroundDetectionRay>()
        .on_insert(|mut world, entity, _| {
            world.commands().entity(entity).insert(
                ShapeCaster::new(
                    Collider::sphere(2.0),
                    {
                        let mut origin = Vec3::ZERO;
                        origin.y -= 1.0;
                        origin
                    },
                    Quat::default(),
                    Dir3::NEG_Y,
                )
                .with_max_distance(0.1),
            );
        });
    world
        .register_component_hooks::<Player>()
        .on_insert(|mut world, entity, _| {
            world.commands().entity(entity).insert(JumpGraceTimer {
                timer: {
                    let mut timer = Timer::from_seconds(0.1, TimerMode::Once);
                    timer.tick(Duration::from_secs(1));
                    timer
                },
            });
        });
}

fn move_player(
    mut player: Query<(
        &mut Transform,
        &mut Player,
        &mut ExternalForce,
        &mut LinearVelocity,
        &ShapeHits,
        &mut JumpGraceTimer,
    )>,
    keys: Res<ButtonInput<KeyCode>>,
    timer: Res<Time>,
) {
    for (mut transform, mut player, mut force, mut velocity, hits, mut jump_grace_timer) in
        &mut player
    {
        jump_grace_timer.timer.tick(timer.delta());
        if keys.pressed(KeyCode::ArrowLeft) {
            transform.rotate_y(2.0 * timer.delta_secs());
        }
        if keys.pressed(KeyCode::ArrowRight) {
            transform.rotate_y(-2.0 * timer.delta_secs());
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
        let grounded = hits.iter().next().is_some()
            && (hits.iter().next().unwrap().normal1.dot(Vec3::Y) > 0.75)
            && jump_grace_timer.timer.finished();
        if grounded {
            movement = (movement.normalize_or_zero() * player.speed)
                .reject_from(hits.iter().next().unwrap().normal1);
            if keys.just_pressed(KeyCode::Space) {
                movement.y += player.jump_height;
                jump_grace_timer.timer.reset();
            }
            if movement != Vec3::ZERO {
                player.was_moving = true;
                let mut x = velocity.x;
                let mut y = velocity.y;
                let mut z = velocity.z;
                for (velocity, &movement) in [
                    (&mut x, &movement.x),
                    (&mut y, &movement.y),
                    (&mut z, &movement.z),
                ] {
                    if velocity.abs() < movement.abs()
                        || *velocity > 0.0 && movement < 0.0
                        || *velocity < 0.0 && movement > 0.0
                        || movement == 0.0
                    {
                        *velocity += movement;
                    }
                }
                velocity.x = x;
                velocity.y = y;
                velocity.z = z;
            } else if player.was_moving {
                velocity.x = 0.0;
                velocity.z = 0.0;
                velocity.y = 0.0;
                player.was_moving = false;
            }
            println!("poo");
        } else {
            movement = movement.normalize_or_zero() * player.air_speed;
            force.apply_force(movement);
            println!("pee");
        }
    }
}

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
pub use components::Player;
use components::*;
mod components;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(Startup, setup)
            .add_systems(
                PhysicsSchedule,
                (ground_check, float_player)
                    .chain()
                    .after(PhysicsStepSet::Last),
            )
            .add_systems(Update, grounded_debug);
    }
}

fn setup(world: &mut World) {
    world
        .register_component_hooks::<Player>()
        .on_add(|mut world, entity, _| {
            let player = world.get::<Player>(entity).unwrap().clone();
            world.commands().entity(entity).insert({
                let rigid_body = RigidBody::Dynamic;
                let collider = Collider::capsule(player.collider_radius, player.collider_height);
                let mut shape_caster_origin = Vec3::ZERO;
                shape_caster_origin.y -= player.collider_height / 2.0;
                let mut ray_caster_origin = Vec3::ZERO;
                ray_caster_origin.y -= player.collider_height / 2.0 + player.collider_radius;
                let shape_caster = ShapeCaster::new(
                    Sphere::new(player.collider_radius),
                    shape_caster_origin,
                    Quat::default(),
                    Dir3::NEG_Y,
                )
                .with_max_distance(player.float_height * 2.0);
                let ray_caster = RayCaster::new(ray_caster_origin, Dir3::NEG_Y)
                    .with_max_distance(player.float_height * 2.0);
                let locked_axes = LockedAxes::new()
                    .lock_rotation_x()
                    .lock_rotation_y()
                    .lock_rotation_z();
                let external_force = ExternalForce::default().with_persistence(false);
                (
                    rigid_body,
                    collider,
                    shape_caster,
                    ray_caster,
                    locked_axes,
                    external_force,
                )
            });
        });
}

fn ground_check(mut query: Query<(&mut PlayerData, &ShapeHits, &RayHits, &Player)>) {
    for (mut player_data, shape_hits, ray_hits, player) in &mut query {
        let mut dot = Option::<f32>::None;
        let mut ray_height = Option::<f32>::None;
        let mut shape_height = Option::<f32>::None;
        for &hit in ray_hits.iter() {
            dot = Some(hit.normal.dot(Vec3::Y));
            ray_height = Some(hit.distance);
        }
        for &hit in shape_hits.iter() {
            shape_height = Some(hit.distance);
        }
        if dot == Some(0.0) {
            dot = Some(1.0);
        }
        match (dot, ray_height, shape_height) {
            (Some(dot), Some(ray_height), Some(shape_height)) => {
                if shape_height <= player.float_height {
                    if dot >= player.max_slope {
                        player_data.grounded = GroundedState::Grounded;
                        player_data.ground_distance = Some(shape_height);
                    } else {
                        player_data.grounded =
                            GroundedState::Ungrounded(UngroundedReason::SteepSlope);
                        player_data.ground_distance = Some(ray_height);
                    }
                } else {
                    player_data.grounded = GroundedState::Ungrounded(UngroundedReason::Airborne);
                    player_data.ground_distance = None;
                }
            }
            _ => {
                player_data.grounded = GroundedState::Ungrounded(UngroundedReason::Airborne);
                player_data.ground_distance = None;
            }
        }
    }
}

fn grounded_debug(mut contexts: EguiContexts, query: Query<&PlayerData>) {
    for player_data in &query {
        egui::Window::new("Grounded").show(contexts.ctx_mut(), |ui| {
            ui.label(format!(
                "{:#?}\n{:#?}",
                player_data.grounded, player_data.ground_distance
            ));
        });
    }
}

fn float_player(mut query: Query<(&Player, &PlayerData, &mut ExternalForce, &LinearVelocity)>) {
    for (player, player_data, mut external_force, velocity) in &mut query {
        match player_data.ground_distance {
            Some(ground_distance) => {
                external_force.apply_force(
                    Dir3::NEG_Y
                        * (((ground_distance - player.float_height) * player.spring_strength)
                            - ((Dir3::NEG_Y.dot(**velocity)) * player.spring_damping)),
                );
            }
            _ => {}
        }
    }
}

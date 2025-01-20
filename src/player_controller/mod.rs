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
                (
                    ground_check,
                    (
                        float_player
                            .ambiguous_with(move_player)
                            .ambiguous_with(push_down_slopes),
                        move_player.ambiguous_with(push_down_slopes),
                        push_down_slopes,
                    ),
                )
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
                let shape_caster_origin = Vec3::ZERO.with_y(player.collider_height / 2.0);
                let shape_caster = ShapeCaster::new(
                    Sphere::new(player.collider_radius * 0.95),
                    shape_caster_origin,
                    Quat::default(),
                    Dir3::NEG_Y,
                );
                let ray_caster = RayCaster::new(Vec3::ZERO, Dir3::NEG_Y).with_max_hits(1);
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

fn ground_check(mut query: Query<(&mut PlayerData, &ShapeHits, &RayHits, &Player, &Transform)>) {
    for (mut player_data, shape_hits, ray_hits, player, transform) in &mut query {
        let mut dot = Option::<f32>::None;
        let mut normal = Option::<Dir3>::None;
        let mut height = Option::<f32>::None;
        let mut hit_point = Option::<Vec3>::None;
        for &hit in ray_hits.iter() {
            if hit.distance == 0.0 {
                normal = Some(Dir3::Y);
                dot = Some(1.0);
                height = Some(player.collider_height / 2.0 - player.collider_radius);
                hit_point = Some(transform.translation - Vec3::Y * height.unwrap());
                break;
            }
            normal = Some(hit.normal.try_into().unwrap());
            dot = Some(hit.normal.dot(Vec3::Y));
            height = Some(hit.distance);
            hit_point = Some(transform.translation - Vec3::Y * height.unwrap());
        }
        if !(height.is_some() && height.unwrap() <= player.float_height) {
            for &hit in shape_hits.iter() {
                normal = Some(hit.normal1.try_into().unwrap());
                dot = Some(hit.normal1.dot(Vec3::Y));
                height = Some(transform.translation.y - hit.point1.y);
                hit_point = Some(hit.point1);
            }
        }

        if height.is_some() && height.unwrap() <= player.float_height {
            if dot.unwrap() >= player.max_slope {
                player_data.grounded = GroundedState::Grounded;
            } else {
                player_data.grounded = GroundedState::Ungrounded(UngroundedReason::SteepSlope);
            }
            player_data.ground_distance = Some(height.unwrap());
            player_data.ground_height = Some(hit_point.unwrap().y);
            player_data.ground_normal = Some(normal.unwrap());
        } else {
            player_data.grounded = GroundedState::Ungrounded(UngroundedReason::Airborne);
            player_data.ground_distance = None;
            player_data.ground_height = None;
            player_data.ground_normal = None;
        }
    }
}

fn grounded_debug(mut contexts: EguiContexts, query: Query<&PlayerData>) {
    for player_data in &query {
        egui::Window::new("Grounded").show(contexts.ctx_mut(), |ui| {
            ui.label(format!(
                "{:#?}\n\n{:#?}\n\n{:#?}",
                player_data.grounded, player_data.ground_distance, player_data.ground_height
            ));
        });
    }
}

fn float_player(
    mut query: Query<(
        &Player,
        &PlayerData,
        &mut ExternalForce,
        &mut LinearVelocity,
        &mut Transform,
        &mut GravityScale,
    )>,
) {
    for (player, player_data, mut external_force, mut velocity, mut transform, mut gravity_scale) in
        &mut query
    {
        match (player_data.ground_distance, player_data.ground_height) {
            (Some(ground_distance), Some(ground_position)) => {
                let target_height = ground_position + player.float_height;
                if transform.translation.y < target_height {
                    gravity_scale.0 = 0.0;
                    velocity.y = 0.0;
                    transform.translation.y = target_height;
                } else {
                    gravity_scale.0 = 0.0;
                }
            }
            _ => {
                gravity_scale.0 = 1.0;
            }
        }
    }
}

fn push_down_slopes(
    mut query: Query<(&ComputedMass, &mut ExternalForce, &PlayerData)>,
    gravity: Res<Gravity>,
) {
    for (mass, mut external_force, player_data) in &mut query {
        match player_data.grounded {
            GroundedState::Ungrounded(UngroundedReason::SteepSlope) => {
                let magnitude = (gravity.0 * mass.value())
                    .reject_from(player_data.ground_normal.unwrap().as_vec3())
                    .length();
                let mut direction = player_data.ground_normal.unwrap().as_vec3();
                direction.y = 0.0;
                external_force.apply_force(direction.normalize() * magnitude);
            }
            _ => {}
        }
    }
}

fn move_player(
    mut query: Query<(&Player, &mut ExternalForce, &Transform)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (player, mut external_force, transform) in &mut query {
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
        movement = movement.normalize_or_zero() * player.grounded_max_speed;
        external_force.apply_force(movement);
    }
}

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
                let mut shape_caster_origin = Vec3::ZERO;
                shape_caster_origin.y -= player.collider_height / 2.0;
                let mut ray_caster_origin = Vec3::ZERO;
                ray_caster_origin.y -= player.collider_height / 2.0 + player.collider_radius;
                let shape_caster = ShapeCaster::new(
                    Sphere::new(player.collider_radius * 0.95),
                    shape_caster_origin,
                    Quat::default(),
                    Dir3::NEG_Y,
                )
                .with_max_distance(player.float_height * 2.0);
                let ray_caster = RayCaster::new(ray_caster_origin, Dir3::NEG_Y)
                    .with_max_distance(player.float_height * 2.0)
                    .with_max_hits(1);
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
        let mut normal = Option::<Dir3>::None;
        let mut shape_hit = Option::<ShapeHitData>::None;
        for &hit in ray_hits.iter() {
            normal = Some(hit.normal.try_into().unwrap());
            dot = Some(hit.normal.dot(Vec3::Y));
        }
        for &hit in shape_hits.iter() {
            shape_hit = Some(hit);
        }
        match (dot, shape_hit) {
            (Some(dot), Some(shape_hit)) => {
                if shape_hit.distance <= player.float_height {
                    if dot >= player.max_slope {
                        player_data.grounded = GroundedState::Grounded;
                        player_data.ground_distance = Some(shape_hit.distance);
                    } else {
                        player_data.grounded = GroundedState::Ungrounded(
                            UngroundedReason::SteepSlope(normal.unwrap()),
                        );
                        player_data.ground_distance = Some(shape_hit.distance);
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

fn push_down_slopes(
    mut query: Query<(&ComputedMass, &mut ExternalForce, &PlayerData)>,
    gravity: Res<Gravity>,
) {
    for (mass, mut external_force, player_data) in &mut query {
        match player_data.grounded {
            GroundedState::Ungrounded(UngroundedReason::SteepSlope(normal)) => {
                let magnitude = (gravity.0 * mass.value()).reject_from(normal.as_vec3()).length();
                let mut direction = normal.as_vec3();
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

//watch very very valet video again for movement lol

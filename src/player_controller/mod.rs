use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::{input::mouse::MouseMotion, window::CursorGrabMode};
use bevy_egui::{EguiContexts, egui};
pub use components::Player;
use components::*;
mod components;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(Startup, (setup, grab_mouse))
            .add_systems(
                PhysicsSchedule,
                (
                    ground_check,
                    (
                        float_player.ambiguous_with_all(),
                        move_player.ambiguous_with_all(),
                        push_down_slopes.ambiguous_with_all(),
                        rotate_player.ambiguous_with_all(),
                        rotate_camera.ambiguous_with_all(),
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

            let child_shape_caster = ShapeCaster::new(
                Collider::capsule(player.collider_radius * 0.95, player.collider_height),
                Vec3::ZERO,
                Quat::default(),
                Dir3::Y,
            )
            .with_query_filter(SpatialQueryFilter::from_excluded_entities([entity]));
            let collider_child = world
                .commands()
                .spawn((child_shape_caster, PlayerShapeCasterChild))
                .id();
            world
                .commands()
                .entity(entity)
                .add_children(&[collider_child]);

            let child_camera = Camera3d::default();
            let child_projection = Projection::from(PerspectiveProjection {
                fov: player.fov.to_radians(),
                ..default()
            });
            let camera_child = world
                .commands()
                .spawn((child_camera, child_projection, PlayerCameraChild::default()))
                .id();
            world
                .commands()
                .entity(entity)
                .add_children(&[camera_child]);
        });
}

fn ground_check(mut query: Query<(&mut PlayerData, &ShapeHits, &RayHits, &Player, &Transform)>) {
    for (mut player_data, shape_hits, ray_hits, player, transform) in &mut query {
        let mut ground_angle = Option::<f32>::None;
        let mut normal = Option::<Dir3>::None;
        let mut height = Option::<f32>::None;
        let mut hit_point = Option::<Vec3>::None;
        for &hit in ray_hits.iter() {
            if hit.distance == 0.0 {
                normal = Some(Dir3::Y);
                ground_angle = Some(0.0);
                height = Some(player.collider_height / 2.0 - player.collider_radius);
                hit_point = Some(transform.translation - Vec3::Y * height.unwrap());
                break;
            }
            normal = Some(hit.normal.try_into().unwrap());
            ground_angle = Some(hit.normal.angle_between(Vec3::Y).to_degrees());
            height = Some(hit.distance);
            hit_point = Some(transform.translation - Vec3::Y * height.unwrap());
        }
        if !(height.is_some() && height.unwrap() <= player.float_height) {
            for &hit in shape_hits.iter() {
                normal = Some(hit.normal1.try_into().unwrap());
                ground_angle = Some(hit.normal1.angle_between(Vec3::Y).to_degrees());
                height = Some(transform.translation.y - hit.point1.y);
                hit_point = Some(hit.point1);
            }
        }
        //rotating sometimes changes Y transform a tiny amount, 0.01 buffer makes that fine
        if height.is_some() && height.unwrap() <= player.float_height + 0.01 {
            //it's just innaccurate sometimes idk
            if ground_angle.unwrap() <= player.max_slope_degrees + 0.1 {
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
    mut query: Query<
        (
            &Player,
            &mut PlayerData,
            &mut LinearVelocity,
            &mut Transform,
            &mut GravityScale,
        ),
        Without<PlayerShapeCasterChild>,
    >,
    child_query: Query<&ShapeHits, With<PlayerShapeCasterChild>>,
    time: Res<Time>,
) {
    for (player, mut player_data, mut velocity, mut transform, mut gravity_scale) in &mut query {
        match player_data.ground_height {
            Some(ground_height) => {
                let (float_height, pushed_down) = float_height(
                    child_query.iter().next().unwrap(),
                    &ground_height,
                    &player.float_height,
                    &(player.collider_height + player.collider_radius * 2.0),
                );
                let target_height = ground_height + float_height;

                if transform.translation.y < target_height {
                    gravity_scale.0 = 0.0;
                    velocity.y = 0.0;
                    match player_data.prev_pushed_down_state {
                        true => {
                            if !pushed_down && player_data.prev_pushed_down_state {
                                transform.translation.y = ((transform.translation.y * Vec3::Y)
                                    .move_towards(
                                        target_height * Vec3::Y,
                                        10.0 * time.delta_secs(),
                                    ))
                                .length();
                                //TODO: rewrite to not do back and forth conversion to vec3
                                if transform.translation.y == target_height {
                                    player_data.prev_pushed_down_state = false;
                                }
                            } else {
                                transform.translation.y = target_height;
                            }
                        }
                        false => {
                            transform.translation.y = target_height;
                            player_data.prev_pushed_down_state = pushed_down;
                        }
                    }
                } else {
                    gravity_scale.0 = 0.0;
                }
            }
            _ => {
                gravity_scale.0 = 1.0;
                player_data.prev_pushed_down_state = false;
            }
        }
    }
}

fn float_height(
    hits: &ShapeHits,
    ground_height: &f32,
    float_height: &f32,
    player_height: &f32,
) -> (f32, PushedDown) {
    let target_height = ground_height + float_height;
    for hit in hits.iter() {
        if hit.point1.y >= target_height + player_height / 2.0 {
            return (*float_height, false);
        } else {
            return (hit.point1.y - ground_height - player_height / 2.0, true);
        }
    }

    (*float_height, false)
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

fn grab_mouse(mut window: Single<&mut Window>) {
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn rotate_player(mut query: Query<(&mut Transform, &Player)>, mut mouse: EventReader<MouseMotion>) {
    for movement in mouse.read() {
        for (mut transform, player) in &mut query {
            transform.rotate_y(-1.0 * movement.delta.x * player.horizontal_camera_sensitivity);
        }
    }
}

fn rotate_camera(
    mut camera_query: Query<(&mut Transform, &mut PlayerCameraChild)>,
    player_query: Query<&Player>,
    mut mouse: EventReader<MouseMotion>,
) {
    for movement in mouse.read() {
        for (mut transform, mut camera) in &mut camera_query {
            for player in &player_query {
                transform.rotation = Quat::default();
                camera.pitch += -1.0 * movement.delta.y * player.vertical_camera_sensitivity;
                camera.pitch = camera.pitch.clamp(-90f32.to_radians(), 90f32.to_radians());
                transform.rotate_around(
                    Vec3::ZERO,
                    Quat::from_rotation_x( camera.pitch),
                );
            }
        }
    }
}

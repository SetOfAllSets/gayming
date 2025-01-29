use avian3d::prelude::*;
use bevy::math::VectorSpace;
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
                FixedUpdate,
                (
                    ground_check,
                    (
                        float_player.ambiguous_with_all(),
                        push_down_slopes.ambiguous_with_all(),
                        rotate_player.ambiguous_with_all(),
                        rotate_camera.ambiguous_with_all(),
                        jump.ambiguous_with_all(),
                        move_player.ambiguous_with_all(),
                        crouch.ambiguous_with_all(),
                    ),
                )
                    .chain()
                    .after(PhysicsStepSet::Last),
            )
            .add_systems(Update, debug);
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
                let shape_caster = ShapeCaster::new(
                    Sphere::new(player.collider_radius * 0.95),
                    Vec3::ZERO,
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

            let mut child_camera = Camera::default();
            child_camera.is_active = false;
            let child_projection = Projection::from(PerspectiveProjection {
                fov: player.fov.to_radians(),
                ..default()
            });
            //let mut temp_transform = Transform::default();
            //temp_transform.translation.z -= 7.0;
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
                height = Some(player.collider_height / 2.0 + player.collider_radius);
                hit_point = Some(transform.translation - Vec3::Y * height.unwrap());
                break;
            }
            normal = Some(hit.normal.try_into().unwrap());
            ground_angle = Some(hit.normal.angle_between(Vec3::Y).to_degrees());
            height = Some(hit.distance);
            hit_point = Some(transform.translation - Vec3::Y * height.unwrap());
        }
        if !(height.is_some() && height.unwrap() <= player.stand_height) {
            for &hit in shape_hits.iter() {
                normal = Some(hit.normal1.try_into().unwrap());
                ground_angle = Some(hit.normal1.angle_between(Vec3::Y).to_degrees());
                height = Some(transform.translation.y - hit.point1.y);
                hit_point = Some(hit.point1);
            }
        }
        // + 0.2 allows us to stay glued to ramps
        if height.is_some()
            && height.unwrap() - (player.collider_radius + player.collider_height / 2.0)
                <= player.stand_height + 0.2
        {
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

fn debug(mut contexts: EguiContexts, query: Query<(&PlayerData, &LinearVelocity)>) {
    for (player_data, velocity) in &query {
        egui::Window::new("Grounded").show(contexts.ctx_mut(), |ui| {
            ui.label(format!(
                "{:#?}\n\n{:#?}\n\n{:#?}",
                player_data.grounded, player_data.ground_distance, player_data.ground_height
            ));
        });
        egui::Window::new("Jumping").show(contexts.ctx_mut(), |ui| {
            ui.label(format!("{:#?}", player_data.jumped));
        });
        egui::Window::new("Speed").show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Speed: {}", velocity.0.length()));
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
    //TODO: Flatten this funcion and generally make it less of an eyesore
    for (player, player_data, mut velocity, mut transform, mut gravity_scale) in &mut query {
        if !player_data.jumped.finished() {
            gravity_scale.0 = 1.0;
            return;
        }
        match player_data.ground_height {
            Some(ground_height) => {
                let target_height = ground_height
                    + player.stand_height
                    + (player.collider_radius + player.collider_height / 2.0);

                if transform.translation.y < target_height {
                    gravity_scale.0 = 0.0;
                    velocity.y = 0.0;

                    transform.translation.y = target_height;
                } else {
                    gravity_scale.0 = 0.0;
                    transform.translation.y = target_height;
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
                let force = Vec3::NEG_Y
                    * (gravity.0 * mass.value())
                        .reject_from(player_data.ground_normal.unwrap().as_vec3())
                        .length();
                external_force
                    .apply_force(force.reject_from(player_data.ground_normal.unwrap().as_vec3()));
            }
            _ => {}
        }
    }
}

fn move_player(
    mut query: Query<(
        &Player,
        &mut ExternalForce,
        &Transform,
        &LinearVelocity,
        &mut PlayerData,
    )>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (player, mut external_force, transform, velocity, mut player_data) in &mut query {
        let mut movement = Vec3::ZERO;
        let mut slowdown = Vec3::ZERO;
        let mut moving = false;
        if keys.pressed(KeyCode::KeyW) {
            movement += transform.forward().as_vec3();
            moving = true;
        } else if player_data.grounded == GroundedState::Grounded
            && velocity.dot(transform.forward().as_vec3()) > 0.0
        {
            slowdown -= transform.forward().as_vec3()
        }
        if keys.pressed(KeyCode::KeyS) {
            movement += transform.back().as_vec3();
            moving = true;
        } else if player_data.grounded == GroundedState::Grounded
            && velocity.dot(transform.back().as_vec3()) > 0.0
        {
            slowdown -= transform.back().as_vec3()
        }
        if keys.pressed(KeyCode::KeyQ) {
            movement += transform.left().as_vec3();
            moving = true;
        } else if player_data.grounded == GroundedState::Grounded
            && velocity.dot(transform.left().as_vec3()) > 0.0
        {
            slowdown -= transform.left().as_vec3()
        }
        if keys.pressed(KeyCode::KeyD) {
            movement += transform.right().as_vec3();
            moving = true;
        } else if player_data.grounded == GroundedState::Grounded
            && velocity.dot(transform.right().as_vec3()) > 0.0
        {
            slowdown -= transform.right().as_vec3()
        }
        if player_data.grounded == GroundedState::Grounded {
            movement = movement.normalize_or_zero() * player.grounded_speed;
            movement += slowdown.normalize_or_zero() * player.ground_friction;
            movement = movement.reject_from(player_data.ground_normal.unwrap().as_vec3());
            if !moving {
                movement = Vec3::new(velocity.0.x, 0.0, velocity.0.z).normalize_or_zero()
                    * -1.0
                    * player.ground_friction;
            }
        } else {
            movement = movement.normalize_or_zero() * player.airborne_sped;
        }

        external_force.apply_force(movement);

        if keys.pressed(KeyCode::KeyC) {
            player_data.crouching = true;
        } else {
            player_data.crouching = false;
        }
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
                transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(camera.pitch));
            }
        }
    }
}

fn jump(
    mut query: Query<(&mut LinearVelocity, &mut PlayerData, &Player)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut velocity, mut player_data, player) in &mut query {
        if keys.pressed(KeyCode::Space)
            && player_data.grounded == GroundedState::Grounded
            && player_data.jumped.finished()
        {
            velocity.y += player.jump_height;
            player_data.jumped.reset();
        }
        player_data.jumped.tick(time.delta());
    }
}

fn crouch(
    mut query: Query<(&PlayerData, &mut Collider, &Player), Without<PlayerShapeCasterChild>>,
    mut child_query: Query<&mut ShapeCaster, With<PlayerShapeCasterChild>>,
) {
    for (player_data, mut collider, player) in &mut query {
        for mut shape_caster in &mut child_query {
            if player_data.crouching {
                *collider = Collider::capsule(player.collider_radius, player.crouch_height);
                *shape_caster = ShapeCaster::new(
                    Collider::capsule(player.collider_radius * 0.95, player.crouch_height),
                    Vec3::ZERO,
                    Quat::default(),
                    Dir3::Y,
                );
            } else {
                *collider = Collider::capsule(player.collider_radius, player.collider_height);
                *shape_caster = ShapeCaster::new(
                    Collider::capsule(player.collider_radius * 0.95, player.collider_height),
                    Vec3::ZERO,
                    Quat::default(),
                    Dir3::Y,
                );
            }
        }
    }
}

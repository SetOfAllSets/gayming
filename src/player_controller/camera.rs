use crate::player_controller::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

pub fn move_camera(
    mut query: Query<(&mut PlayerCameraChild, &mut Transform)>,
    player_query: Query<&Player>,
    mut scroll: EventReader<MouseWheel>,
    mut movement: EventReader<MouseMotion>,
) {
    for player in &player_query {
        for (mut camera_child, mut transform) in &mut query {
            for movement in movement.read() {
                camera_child.pitch -= movement.delta.y * player.vertical_camera_sensitivity;
                camera_child.pitch = camera_child
                    .pitch
                    .clamp(-90f32.to_radians(), 90f32.to_radians());
            }
            for scroll in scroll.read() {
                camera_child.distance -= scroll.y;
                camera_child.distance = camera_child.distance.clamp(0.0, 15.0);
            }
            let mut camera_position = Vec2::from_angle(-1.0 * camera_child.pitch)
                .extend(0.0)
                .zyx();
            camera_position *= camera_child.distance;
            transform.translation = camera_position;
            transform.rotation = Quat::default();
            let translation = transform.translation;
            transform.rotate_around(translation, Quat::from_rotation_x(camera_child.pitch));
        }
    }
}

pub fn rotate_player(
    mut query: Query<(&mut Transform, &Player)>,
    mut movement: EventReader<MouseMotion>,
) {
    for (mut transform, player) in &mut query {
        for movement in movement.read() {
            transform.rotate(Quat::from_rotation_y(
                -1.0 * movement.delta.x * player.horizontal_camera_sensitivity,
            ));
        }
    }
}

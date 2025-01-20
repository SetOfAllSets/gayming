use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
#[require(
    PlayerData,
    RigidBody,
    Collider,
    ShapeCaster,
    LockedAxes,
    RayCaster,
    ExternalForce
)]
pub struct Player {
    pub collider_height: f32,
    pub collider_radius: f32,
    pub max_slope: f32,
    pub float_height: f32,
    pub spring_strength: f32,
    pub spring_damping: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            collider_height: 2.0,
            collider_radius: 2.0,
            max_slope: 0.75,
            float_height: 2.0,
            spring_strength: 1000.0,
            spring_damping: 75.0,
        }
    }
}

#[derive(Component, Reflect, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct PlayerData {
    pub grounded: GroundedState,
    pub ground_distance: Option<f32>,
}

impl Default for PlayerData {
    fn default() -> Self {
        PlayerData {
            grounded: GroundedState::Ungrounded(UngroundedReason::Airborne),
            ground_distance: None,
        }
    }
}

#[derive(Reflect, Debug, PartialEq)]
pub enum GroundedState {
    Grounded,
    Ungrounded(UngroundedReason),
}

#[derive(Reflect, Debug, PartialEq)]
pub enum UngroundedReason {
    Airborne,
    SteepSlope,
}

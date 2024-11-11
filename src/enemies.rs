use bevy::{ecs::query, math::NormedVectorSpace, prelude::*};
use bevy_rapier2d::prelude::*;

use serde::Deserialize;

use crate::{scorch::Scorch, blocks::BlockInfo};

pub struct EnemyPlugin;

const ENEMY_FORCE_STRENGTH: f32 = 999.0;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy_info: EnemyInfo,
}

#[allow(dead_code)]
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, enemy_movement_system)
            //.add_systems(Startup, setup_test_enemy)
        ;
    }
}

/// Information on enemies
#[derive(Component, Debug, Clone, Copy, Deserialize)]
#[allow(dead_code)]
pub struct EnemyInfo {
    pub e_type: EnemyType,
    pub health: f32,
    pub move_speed: f32,
    pub dmg: f32,
}

impl EnemyInfo {
    pub fn contact_dmg(&self) -> f32 {
        self.dmg
    }

    pub fn speed(&self) -> f32 {
        self.move_speed
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum EnemyType {
    RunDown,
    Ranged,
    Summoner,
}

// TIL: crashed when used mutable transform remember this
fn enemy_movement_system(
    //commands: Commands,
    mut enemy_query: Query<(&mut ExternalImpulse, &Transform, &mut EnemyInfo)>,
    scorch_query: Query<&Transform, With<Scorch>>,
) {
    // takes the transform from scorch, and maps the 2d+z space into a 2d space
    if let Ok(scorch_pos) = scorch_query.get_single()
        .and_then(|scor_tran| Ok(scor_tran.translation.truncate())) 
    {
        for (mut e_imp, e_trans, e_info) in enemy_query.iter_mut() {
            if e_info.e_type == EnemyType::RunDown {
                // move towards scorch
            e_imp.impulse += (scorch_pos - e_trans.translation.truncate()).normalize()
            * ENEMY_FORCE_STRENGTH * e_info.speed();
            }
        }
    }
}

pub fn spawn_enemy(
    commands: &mut Commands,
    e_pos: Vec2,
    e_info: EnemyInfo,
) {
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::ball(25.0),
            Restitution::coefficient(0.5),
            TransformBundle::from(Transform::from_xyz(e_pos.x, e_pos.y, 0.0)),
            ExternalImpulse::default(),
            GravityScale(0.0),
            ColliderMassProperties::Density(1.0),
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
            e_info,
        ));
}
use bevy::{ecs::query, math::NormedVectorSpace, prelude::*};
use bevy_rapier2d::prelude::*;

use serde::Deserialize;

use crate::{scorch::Scorch, blocks::BlockInfo};
pub struct EnemyPlugin;

const ENEMY_FORCE_STRENGTH: f32 = 99999.0;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy_info: EnemyInfo,
}

#[allow(dead_code)]
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, enemy_movement_system)
            .add_systems(Startup, setup_test_enemy)
        ;
    }
}

/// Information on enemies
#[derive(Component, Debug, Clone, Copy, Deserialize)]
#[allow(dead_code)]
pub struct EnemyInfo {
    health: f32,
    speed: f32,
}

// impl EnemyInfo {
//     //pub fn move(&mut self, )
// }

fn enemy_movement_system(
    //commands: Commands,
    mut enemy_query: Query<(&mut ExternalImpulse, &mut Transform, &mut EnemyInfo)>,
    scorch_query: Query<&Transform, With<Scorch>>,
) {
    // takes the transform from scorch, and maps the 2d+z space into a 2d space
    if let Ok(scorch_pos) = scorch_query.get_single()
        .and_then(|scor_tran| Ok(scor_tran.translation.truncate())) 
    {
        for (mut e_imp, e_trans, e_info) in enemy_query.iter_mut() {
            // move towards scorch
            e_imp.impulse += (scorch_pos - e_trans.translation.truncate()).normalize()
                    * ENEMY_FORCE_STRENGTH * e_info.speed;
        }
    }

    
}

fn setup_test_enemy(mut commands: Commands) {
    // commands
    //     .spawn((
    //         RigidBody::Dynamic,
    //         Collider::ball(25.0),
    //         Restitution::coefficient(0.5),
    //         TransformBundle::from(Transform::from_xyz(100.0, 0.0, 0.0)),
    //         ExternalImpulse::default(),
    //         //Velocity::default(),
    //         GravityScale(0.0),
    //         ColliderMassProperties::Density(1.0),
    //         LockedAxes::ROTATION_LOCKED,
    //         // Damping {
    //         //     linear_damping: 0.1, 
    //         //     angular_damping: 0.0
    //         // },
    //         EnemyInfo {
    //             health: 100.0,
    //             speed: 10.0,
    //         },
    //         ActiveEvents::COLLISION_EVENTS,
    //         //Friction::coefficient(0.0),
    //     ));
}
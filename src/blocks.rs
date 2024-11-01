use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::Deserialize;

pub struct BlockPlugin;

#[derive(Bundle)]
pub struct BlockBundle {
    pub block_info: BlockInfo,
}

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_blocks);
    }
}

/// This is an struct for information on the burn type for a block
#[derive(Component, Debug, Clone, Copy, Deserialize)]
#[allow(dead_code)]
pub struct BlockInfo {
    /// If this can be set on fire
    pub burnable:       bool,
    /// When burnt can it be put out
    pub extinguishable: bool,
    /// (How long it will burn, when it starts burning (preburn == f64::MAX))
    pub burn_time:      (f32, f32),
    // currently dont use pos and size
    // /// position of the block
    // pub pos:            Vec2,
    // /// size of the block
    // pub size:           Vec2,
    //TODO slants, movable, explosive
}

impl BlockInfo {
    #[allow(dead_code)]
    pub fn default(self) -> BlockInfo {
        BlockInfo {
            burnable:       true,
            extinguishable: true,
            burn_time:      (10.0, 0.0),
        }
    }
    
    pub fn new(burn: bool, exti: bool, btime: f32) -> BlockInfo {
        BlockInfo {
            burnable:       burn,
            extinguishable: exti,
            burn_time:      (btime, 0.0),
        }
    }
    
    pub fn set_burn(&mut self, start_time: f32) {
        self.burn_time.1 = start_time;
    }
}

fn setup_blocks(
    mut commands: Commands,
) {
    // this is the platform 
    // commands
    //     .spawn((
    //         // RigidBody::Dynamic,
    //         Collider::cuboid(500.0, 50.0),
    //         TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)),
    //         BlockInfo::new(
    //             true, 
    //             false, 
    //             0.1, 
    //         ),
    //         ActiveEvents::COLLISION_EVENTS,
    //         //Friction::coefficient(2.0),
    //     ));
}
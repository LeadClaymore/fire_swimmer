use bevy::prelude::*;
//use bevy_rapier2d::prelude::*;

use serde::Deserialize;

pub struct BlockPlugin;

#[derive(Bundle)]
pub struct BlockBundle {
    pub block_info: BlockInfo,
}

#[allow(dead_code)]
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, block_burning_system)
        ;
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
    
    #[allow(dead_code)]
    pub fn new(burn: bool, exti: bool, btime: f32) -> BlockInfo {
        BlockInfo {
            burnable:       burn,
            extinguishable: exti,
            burn_time:      (btime, 0.0),
        }
    }
    
    pub fn set_burn(&mut self, start_time: f32) {
        if self.burnable {
            self.burn_time.1 = start_time;
        }
    }
}

fn block_burning_system (
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &BlockInfo)>
) {
    let current_time = time.elapsed_seconds();
    for (entity, info) in query.iter() {
        if info.burn_time.1 != 0.0 {
            if current_time - info.burn_time.1 >= info.burn_time.0 {
                //TODO for now it just despawns, later it might do more
                commands.entity(entity).despawn();
                //println!("Burn timer started for block!");
            }
        }
    }
}
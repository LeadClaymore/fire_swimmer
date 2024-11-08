// default includes
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

// block related aspects
mod blocks;
use blocks::BlockPlugin;
// camera related aspects
mod camera;
use camera::CameraPlugin;
// main charater scorch
mod scorch;
use scorch::ScorchPlugin;
use scorch::Scorch;
// ember
mod ember;
use ember::EmberPlugin;
// collider
mod coll;
use coll::CollPlugin;
// rand
mod rng;
use rng::RngPlugin;
// sdtogame
mod sd_to_game;
use sd_to_game::SdPlugin;
// enemies
mod enemies;
use enemies::EnemyPlugin;

fn main() {
    App::new()
        // built in plugins
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(FpsOverlayPlugin::default())
    
        // home made plugins
        .add_plugins(BlockPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(ScorchPlugin)
        .add_plugins(EmberPlugin)
        .add_plugins(CollPlugin)
        .add_plugins(RngPlugin)
        .add_plugins(SdPlugin)
        .add_plugins(EnemyPlugin)
        // TODO move to a scheduling system
        .run();
}
//end
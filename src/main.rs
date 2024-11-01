// default includes
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

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

fn main() {
    App::new()
        // default plugins
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        // plugins
        .add_plugins(BlockPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(ScorchPlugin)
        .add_plugins(EmberPlugin)
        .add_plugins(CollPlugin)
        .add_plugins(RngPlugin)
        .add_plugins(SdPlugin)
        // TODO move to a scheduling system
        .run();
}
//end
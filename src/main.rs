use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;

// block related aspects
mod blocks;
//use blocks::BlockInfo;
use blocks::BlockPlugin;

// camera related aspects
mod camera;
//use camera::MainCamera;
use camera::CameraPlugin;

// main charater scorch
mod scorch;
use scorch::ScorchPlugin;
use scorch::Scorch;

// ember
mod ember;
use ember::EmberPlugin;
//use ember::EmberComponent;

mod coll;
use coll::CollPlugin;

#[derive(Resource)]
pub struct RngResource {
    pub rng: rand::rngs::SmallRng,
}

impl Default for RngResource {
    fn default() -> Self {
        Self { rng: SmallRng::from_entropy(), }
    }
}

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
        // resources
        .insert_resource(RngResource::default())
        // TODO move to a scheduling system
        //.add_systems(Update, (block_burning_system, collision_event_system))
        .run();
}
//end
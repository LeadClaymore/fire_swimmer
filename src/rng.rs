use bevy::prelude::*;
//use bevy_rapier2d::prelude::*;
// external stuff
use rand::SeedableRng;
use rand::rngs::SmallRng;

// elsewhere in the project

#[derive(Bundle)]
pub struct RngBundle {
    // unused
}

pub struct RngPlugin;

impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        app
            // resources
            .insert_resource(RngResource::default())
        ;
    }
}

#[derive(Resource)]
pub struct RngResource {
    pub rng: rand::rngs::SmallRng,
}
impl Default for RngResource {
    fn default() -> Self {
        Self { rng: SmallRng::from_entropy(), }
    }
}
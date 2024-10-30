use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;

// block related aspects
mod blocks;
use blocks::BlockInfo;
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
use ember::EmberComponent;

#[derive(Resource)]
pub struct RngResource {
    pub rng: rand::rngs::SmallRng,
}

impl Default for RngResource {
    fn default() -> Self {
        Self { rng: SmallRng::from_entropy(), }
    }
}

fn block_burning_system (
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &BlockInfo)>
) {
    let current_time = time.elapsed_seconds();
    for (entity, info) in query.iter() {
        if info.burn_time.1 != f32::MAX {
            if current_time - info.burn_time.1 >= info.burn_time.0 {
                //TODO for now it just despawns, later it might do more
                commands.entity(entity).despawn();
                //println!("Burn timer started for block!");
            }
        }
    }
}

fn collision_event_system (
    //mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    time: Res<Time>,
    mut binfo_query: Query<(Entity, &mut BlockInfo)>,
    ember_query: Query<Entity, With<EmberComponent>>,
    //mut query: Query<(&mut ActiveCollisionTypes, &mut BlockInfo)>,
) {
    for cevent in collision_events.read() {
        //println!("Collision!");
        match cevent {
            CollisionEvent::Started(ent1, ent2, _) => {
                // check if 
                if let Ok((_block_ent, mut binfo)) = binfo_query.get_mut(*ent1) {
                    if ember_query.get(*ent2).is_ok() {
                        //println!("ent1 is a block, and ent2 is a ember!");
                        if binfo.burn_time.1 == f32::MAX {
                            //println!("burn started! {}, {}, {}", binfo.burn_time.0, binfo.burn_time.1, time.elapsed_seconds());
                            binfo.set_burn(time.elapsed_seconds());
                            //println!("burn started! {}, {}, {}", binfo.burn_time.0, binfo.burn_time.1, time.elapsed_seconds());
                        }
                    }
                }
                // same but in reverse
                else if let Ok((_block_ent, mut binfo)) = binfo_query.get_mut(*ent2) {
                    if ember_query.get(*ent1).is_ok() {
                        //println!("ent2 is a block, and ent1 is a ember!");
                        if binfo.burn_time.1 == f32::MAX {
                            //println!("burn started! {}, {}, {}", binfo.burn_time.0, binfo.burn_time.1, time.elapsed_seconds());
                            binfo.set_burn(time.elapsed_seconds());
                            //println!("burn started! {}, {}, {}", binfo.burn_time.0, binfo.burn_time.1, time.elapsed_seconds());
                        }
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => {
                //currently unused for collisions, but it was in the example
            }
        }
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
        // resources
        .insert_resource(RngResource::default())
        // TODO move to a scheduling system
        .add_systems(Update, (block_burning_system, collision_event_system))
        .run();
}
//end
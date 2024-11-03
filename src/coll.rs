use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// elsewhere in the project
use crate::{ember::EmberComponent, blocks::BlockInfo};

#[derive(Bundle)]
pub struct CollBundle {
    debug_component: DebugComp,
}

pub struct CollPlugin;

impl Plugin for CollPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, collision_event_system)
        ;
    }
}

#[derive(Component)]
pub struct DebugComp;

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
                        if binfo.burnable && binfo.burn_time.1 == 0.0 {
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
                        if binfo.burnable && binfo.burn_time.1 == 0.0 {
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
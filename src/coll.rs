use bevy::{ecs::observer::TriggerTargets, prelude::*};
use bevy_rapier2d::prelude::*;

// elsewhere in the project
use crate::{
    blocks::BlockInfo, ember::EmberComponent, scorch::Scorch
};

#[derive(Bundle)]
pub struct CollBundle {
    debug_component: DebugComp,
}

pub struct CollPlugin;

impl Plugin for CollPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, collision_event_system)
            .add_systems(Update, scorch_collision)
        ;
    }
}

#[derive(Component)]
pub struct DebugComp;

fn collision_event_system (
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    time: Res<Time>,
    mut binfo_query: Query<(Entity, &mut BlockInfo)>,
    ember_query: Query<Entity, With<EmberComponent>>,
    mut scorch_query: Query<(Entity, &mut Scorch)>,
    //mut query: Query<(&mut ActiveCollisionTypes, &mut BlockInfo)>,
) {
    //TODO need to read up on collision handling, also change how it is
    for cevent in collision_events.read() {
        match cevent {
            CollisionEvent::Started(ent1, ent2, _) => {
                // check if 
                if let Ok((_block_ent, mut binfo)) = binfo_query.get_mut(*ent1) {
                    if ember_query.get(*ent2).is_ok() {
                        if binfo.burnable && binfo.burn_time.1 == 0.0 {
                            binfo.set_burn(time.elapsed_seconds());
                        }
                    }
                }
                // same but in reverse
                else if let Ok((_block_ent, mut binfo)) = binfo_query.get_mut(*ent2) {
                    if ember_query.get(*ent1).is_ok() {
                        if binfo.burnable && binfo.burn_time.1 == 0.0 {
                            binfo.set_burn(time.elapsed_seconds());
                        }
                    }
                }
                // // if one is scorch and another is ember, then remove the ember and gain some flame
                // else if let Ok((_scor_ent, mut scor_data)) = scorch_query.get_mut(*ent1) {
                //     if ember_query.get(*ent2).is_ok() {
                //         commands.entity(*ent2).despawn();
                //         scor_data.regen_flame();
                //     }
                // }
                // // if one is scorch and another is ember, then remove the ember and gain some flame
                // else if let Ok((_scor_ent, mut scor_data)) = scorch_query.get_mut(*ent2) {
                //     if ember_query.get(*ent1).is_ok() {
                //         commands.entity(*ent1).despawn();
                //         scor_data.regen_flame();
                //     }
                // }
            }
            CollisionEvent::Stopped(_, _, _) => {
                //currently unused for collisions, but it was in the example
            }
        }
    }
}

fn scorch_collision (
    mut commands: Commands,
    rc: Res<RapierContext>,
    mut scor_query: Query<(Entity, &mut Scorch)>,
    emb_query: Query<(), With<EmberComponent>>,
) {
    let (s_entity, mut s_compo) = scor_query.single_mut();
    for co_pair in rc.contact_pairs_with(s_entity) {
        let coll_entity = if co_pair.collider1() == s_entity {
            co_pair.collider2()
        } else {
            co_pair.collider1()
        };
        if emb_query.get(coll_entity).is_ok() {
            commands.entity(coll_entity).despawn();
            s_compo.regen_flame();
        }
    }
}
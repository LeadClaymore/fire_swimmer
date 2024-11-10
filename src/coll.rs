use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// elsewhere in the project
use crate::{
    blocks::BlockInfo, 
    ember::EmberComponent, 
    scorch::Scorch,
    enemies::EnemyInfo,
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
    mut collision_events: EventReader<CollisionEvent>,
    time: Res<Time>,
    mut binfo_query: Query<(Entity, &mut BlockInfo)>,
    ember_query: Query<Entity, With<EmberComponent>>,
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
            }
            CollisionEvent::Stopped(_, _, _) => {
                //currently unused for collisions, but it was in the example
            }
        }
    }
}

// collisions with scorch
fn scorch_collision (
    mut commands: Commands,
    rc: Res<RapierContext>,
    time: Res<Time>,

    // querys for the possible collisions
    mut scor_query: Query<(Entity, &mut Scorch)>,
    emb_query: Query<(), With<EmberComponent>>,
    mut block_query: Query<&mut BlockInfo>,
    mut enemy_query: Query<&mut EnemyInfo>,
) {
    let (s_entity, mut s_compo) = scor_query.single_mut();
    for co_pair in rc.contact_pairs_with(s_entity) {
        // get collisions with scorch
        let coll_entity = if co_pair.collider1() == s_entity {
            // the other colider is either 1 or 2 so we check which one
            co_pair.collider2()
        } else {
            co_pair.collider1()
        };

        // effects on the other entity depending on what it is
        // when an ember
        if emb_query.get(coll_entity).is_ok() {
            commands.entity(coll_entity).despawn();
            s_compo.regen_flame();
        // when a block
        } else if let Ok(mut b_info) = block_query.get_mut(coll_entity) {
            b_info.set_burn(time.elapsed_seconds());
        } else if let Ok(e_info) = enemy_query.get_mut(coll_entity) {
            s_compo.damage_flame(e_info.dmg);
            println!("Health: {}", s_compo.curr_flame);
        }
    }
}
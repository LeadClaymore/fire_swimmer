use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// elsewhere in the project
use crate::{
    blocks::BlockInfo, 
    ember::EmberComponent, 
    scorch::Scorch,
    enemies::EnemyInfo,
    enemies::ProjectileType,
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

pub struct GameCollisionGroups;
impl GameCollisionGroups {
    // these are the collision groups notice, how their bit values do not overlap.
    // this is because the collision event is an addition of the other groups it can collide with
    pub const GROUP_1: u32 = 0b0000_0000_0000_0001; //Scorch
    pub const GROUP_2: u32 = 0b0000_0000_0000_0010; //Embers
    pub const GROUP_3: u32 = 0b0000_0000_0000_0100; //Blocks
    pub const GROUP_4: u32 = 0b0000_0000_0000_1000; //Enemies
    pub const GROUP_5: u32 = 0b0000_0000_0001_0000; //Enemy Projectiles
}

#[derive(Component)]
pub struct DebugComp;



// old code after collision groups added
fn collision_event_system (
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    time: Res<Time>,

    mut scor_query: Query<(Entity, &mut Scorch)>,
    mut binfo_query: Query<&mut BlockInfo>,
    ember_query: Query<Entity, With<EmberComponent>>,
    mut proj_query: Query<&mut ProjectileType>,
) {
    //scorch data
    let (s_entity, mut s_compo) = scor_query.single_mut();

    //TODO so this is how you are meant to do collisions but I should implement collision flags in the Started
    for c_event in collision_events.read() {
        match c_event {
            CollisionEvent::Started(ent1, ent2, _) => {
                // check if 
                if let Ok(mut binfo) = binfo_query.get_mut(*ent1) {
                    if ember_query.get(*ent2).is_ok() {
                        if binfo.burnable && binfo.burn_time.1 == 0.0 {
                            binfo.set_burn(time.elapsed_seconds());
                        }
                    }
                }
                // same but in reverse
                else if let Ok(mut binfo) = binfo_query.get_mut(*ent2) {
                    if ember_query.get(*ent1).is_ok() {
                        if binfo.burnable && binfo.burn_time.1 == 0.0 {
                            binfo.set_burn(time.elapsed_seconds());
                        }
                    }
                }
                //projectile collisions and reverse
                else if let Ok(mut p_info) = proj_query.get_mut(*ent1) {
                    if s_entity == *ent2 {
                        // check if the other entity is scorch
                        // this is because there could be overlap with the collisions if this was in the scorch collisions.
                        //TODO might add a projectile collison class
                        s_compo.damage_flame(p_info.get_dmg(), time.elapsed_seconds());
                    }
                    commands.entity(*ent1).despawn();
                    //TODO when more projectile types exist handle their interactions
                    //TODO this currently might despawn the projectile if this handles the collision before scorch
                } else if let Ok(mut p_info) = proj_query.get_mut(*ent2) {
                    // same as above
                    if s_entity == *ent1 {
                        s_compo.damage_flame(p_info.get_dmg(), time.elapsed_seconds());
                    }
                    commands.entity(*ent2).despawn();
                }
            }
            CollisionEvent::Stopped(_, _, _) => {
                //currently unused, If I wanted to do something when something stops colliding it would be here
            }
        }
    }
}

/// collisions with scorch
fn scorch_collision (
    mut commands: Commands,
    rc: Res<RapierContext>,
    time: Res<Time>,

    // querys for the possible collisions
    mut scor_query: Query<(Entity, &mut Scorch)>,
    emb_query: Query<(), With<EmberComponent>>,
    mut block_query: Query<&mut BlockInfo>,
    mut enemy_query: Query<&mut EnemyInfo>,
    mut proj_query: Query<&mut ProjectileType>,
) {
    //scorch data
    let (s_entity, mut s_compo) = scor_query.single_mut();

    // collisions with scorch
    for co_pair in rc.contact_pairs_with(s_entity) {
        // get collisions with scorch
        let coll_entity = if co_pair.collider1() == s_entity {
            // the other colider is either 1 or 2 so we check which one
            co_pair.collider2()
        } else {
            co_pair.collider1()
        };

        // effects on the other entity depending on what it is
        // when an ember absorb the ember and heal from it
        if emb_query.get(coll_entity).is_ok() {
            commands.entity(coll_entity).despawn();
            s_compo.regen_flame();

        // when in contact with a block, try to burn
        } else if let Ok(mut b_info) = block_query.get_mut(coll_entity) {
            b_info.set_burn(time.elapsed_seconds());
        
        // when in contact with an enemy take damage by contact damage of the enemy
        } else if let Ok(e_info) = enemy_query.get_mut(coll_entity) {
            if s_compo.damage_flame(e_info.contact_dmg(), time.elapsed_seconds()) {
                //TODO add push back need to understand collisions better first tho
            }
        }
        
        // when in contact with a projectile
        // else if let Ok(p_info) = proj_query.get_mut(coll_entity) {
        //     s_compo.damage_flame(p_info.get_dmg(), time.elapsed_seconds());
        //     commands.entity(coll_entity).despawn();
        // }
    }
}
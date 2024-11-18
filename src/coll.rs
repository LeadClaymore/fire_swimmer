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
            .add_systems(Update, collision_handling)
        ;
    }
}

// TODO change the physics of the entier game to mean this does not need to be 100k
/// a modifier added to impulses to move the charater
const FORCE_STRENGTH: f32 = 99999.9;

#[derive(Component)]
pub struct DebugComp;

fn collision_handling(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    time: Res<Time>,

    mut scorch_query: Query<&mut Scorch>,
    mut ember_query: Query<&mut EmberComponent>,
    mut block_query: Query<&mut BlockInfo>,
    mut enemy_query: Query<&mut EnemyInfo>,
    mut e_proj_query: Query<&mut ProjectileType>,

    cg_query: Query<&CollisionGroups>,
    tf_query: Query<&Transform>,
    mut imp_query: Query<&mut ExternalImpulse>,
) {
    //TODO I need to implement setting entitys to be despawned at the end of the frame, rather then mid collision.
    // Duplicate collisions cause warnings and might be an issue later
    //TODO I should implement collision flags in the Started
    for c_event in collision_events.read() {
        match c_event {
            CollisionEvent::Started(e1, e2, _) => {
                // this gets the bits from the collision group of the entity, because all should have them
                if let (Ok(e1_bits), Ok(e2_bits)) = (
                    cg_query.get(*e1).map(|cg| cg.memberships.bits()), 
                    cg_query.get(*e2).map(|cg| cg.memberships.bits())
                ) {
                    // this orders e1 and e2 by the bits in their collision group (lower bits first)
                    //TODO if I start adding entities with more then 1 group membership see if this still works
                    let (e1, e2) = if e1_bits >= e2_bits { (*e2, *e1) } else { (*e1, *e2) };
                    
                    // if e1 is scorch
                    if let Ok(mut s_info) = scorch_query.get_mut(e1) {
                        //println!("collisions are happening with scorch");
                        //scorch ember collision
                        if let Ok(_em_info) = ember_query.get_mut(e2) {
                            //println!("scorch ember collision");
                            commands.entity(e2).despawn();
                            s_info.regen_flame();
                        //scorch block collision
                        } else if let Ok(mut b_info) = block_query.get_mut(e2) {
                            //println!("scorch block collision");
                            if b_info.burnable && b_info.burn_time.1 == 0.0 {
                                b_info.set_burn(time.elapsed_seconds());
                            }
                        //scorch enemy collision
                        } else if let Ok(mut en_info) = enemy_query.get_mut(e2) {
                            //println!("scorch enemy collision");
                            // this makes scorch take damage //TODO might want to make the enemies take damage too
                            if s_info.damage_flame(en_info.contact_dmg(), time.elapsed_seconds()) {
                                if let Some(dir) = get_dir_to(e1, e2, &tf_query) {
                                    entity_knockback(e1, &mut imp_query, dir);
                                    entity_knockback(e2, &mut imp_query, -dir);
                                    //TODO I need some form of movement stun on knockback
                                    // bc enemies just keep movin forward
                                } else {
                                    println!("Error with scorch enemy collision dirrection handling")
                                }
                            }

                            // TODO this needs to happen every frame not just on contact
                            // contact dmg against enemies
                            if en_info.take_dmg(10.0) {
                                //this happens when the enemy is dead
                                commands.entity(e2).despawn();
                            }
                            
                        //scorch projectile collision
                        } else if let Ok(p_info) = e_proj_query.get_mut(e2) {
                            //println!("scorch projectile collision");
                            // when scorch collides with an projectile it takes damage and the proj despawns
                            s_info.damage_flame(p_info.get_dmg(), time.elapsed_seconds());
                            commands.entity(e2).despawn();
                        } else {
                            println!("ERROR: scorch unknown collision {:b}, {:b}", e1_bits, e2_bits);
                        }
                    // if e1 is ember
                    } else if let Ok(mut _em_info) = ember_query.get_mut(e1) {
                        //println!("collisions are happening with scorch");
                        if let Ok(mut b_info) = block_query.get_mut(e2) {
                            //println!("ember block collision");
                            if b_info.burnable && b_info.burn_time.1 == 0.0 {
                                b_info.set_burn(time.elapsed_seconds());
                            }
                        } else if let Ok(mut en_info) = enemy_query.get_mut(e2) {
                            //println!("ember enemy collision");
                            //TODO embers currently deal 10 dmg independent on flame level
                            if en_info.take_dmg(10.0) {
                                //this happens when the enemy is dead
                                commands.entity(e2).despawn();
                            }
                            commands.entity(e1).despawn();

                        // ember projectile collision
                        } else if let Ok(p_info) = e_proj_query.get_mut(e2) {
                            //println!("ember projectile collision");
                        } else {
                            println!("ERROR: ember unknown collision {:b}, {:b}", e1_bits, e2_bits);
                        }
                    }
                } else {
                    println!("Error in getting collision group from entity for collision");
                }
            }
            CollisionEvent::Stopped(_, _, _) => {
                //currently unused, If I wanted to do something when something stops colliding it would be here
            }
        }
    }
}

fn get_dir_to(
    e1: Entity,
    e2: Entity,
    tf_query: &Query<&Transform>,
) -> Option<Vec2> {
    if let Ok(pos1) = tf_query.get(e1).map(|t1| t1.translation.truncate()) {
        if let Ok(pos2) = tf_query.get(e2).map(|t2| t2.translation.truncate()) {
            return Some((pos1 - pos2).normalize());
        }
    }
    None
}

fn entity_knockback (
    ent: Entity,
    imp_query: &mut Query<&mut ExternalImpulse>,
    direction: Vec2,
) {
    if let Ok(mut e_imp) = imp_query.get_mut(ent) {
        e_imp.impulse = direction * 100.0 * FORCE_STRENGTH;
        //println!("Knockback happened");
    } else {
        println!("Error with knockback with entiy {:?}", ent);
    }
}
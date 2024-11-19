use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
// external stuff
// elsewhere in the project
use crate::blocks::BlockInfo;
use crate::enemies::{spawn_enemy, EnemyInfo};

#[derive(Bundle)]
pub struct SdBundle {
    // unused
}

pub struct SdPlugin;

impl Plugin for SdPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_from_json)
            //.add_systems(Startup, spawn_enemies_from_json)
        ;
    }
}

#[derive(Deserialize)]
struct LevelData {
    blocks: Vec<BlockData>,
    enemies: Vec<EnemyData>,
}

#[derive(Deserialize)]
struct BlockData {
    pos: [f32; 2],
    size: [f32; 2],
    block_info: BlockInfo,
}

#[derive(Deserialize)]
struct EnemyData {
    pos: [f32; 2],
    size: f32,
    e_info: EnemyInfo,
}

fn spawn_from_json(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Open the JSON file
    let file = File::open("levels/lv1.json").expect("Cannot open lv1.json");
    let reader = BufReader::new(file);

    // Deserialize the JSON into a Vec<BlockData>
    let data: LevelData = serde_json::from_reader(reader).expect("Error parsing lv1.json");

    for enemy in data.enemies {
        spawn_enemy(
            &mut commands, 
            Vec2::from(enemy.pos),
            enemy.e_info,
            enemy.size,
        );
    }
    

    for block in data.blocks {
        //TODO I need this to only call once instead of for every block
        //let t_block = asset_server.load("assets/t_block.png");

        commands
            .spawn((
                Collider::cuboid(block.size[0], block.size[1]),
                TransformBundle::from(Transform::from_xyz(block.pos[0], block.pos[1], 0.0)),
                block.block_info,

                // SpriteBundle {
                //     texture: t_block,
                //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
                //     ..Default::default()
                // },
                CollisionGroups::new(
                    // G1 is Scorch, G2 is embers, G3 is blocks, G4 is enemies, G5 is enemy_projectiles
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_4 | Group::GROUP_5,
                ),
                ActiveEvents::COLLISION_EVENTS,
            ));
    }
}
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
// external stuff
// elsewhere in the project
use crate::blocks::BlockInfo;

#[derive(Bundle)]
pub struct SdBundle {
    // unused
}

pub struct SdPlugin;

impl Plugin for SdPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_blocks_from_json)
        ;
    }
}

#[derive(Deserialize)]
struct BlockData {
    pos: [f32; 2],
    size: [f32; 2],
    //blockInfo: 
}

fn spawn_blocks_from_json(
    mut commands: Commands,
) {
    // Open the JSON file
    let file = File::open("levels/lv1.json").expect("Cannot open lv1.json");
    let reader = BufReader::new(file);

    // Deserialize the JSON into a Vec<BlockData>
    let blocks: Vec<BlockData> = serde_json::from_reader(reader).expect("Error parsing lv1.json");

    for block in blocks {
        commands
            .spawn((
                // RigidBody::Dynamic,
                Collider::cuboid(block.size[0], block.size[1]),
                TransformBundle::from(Transform::from_xyz(block.pos[0], block.pos[1], 0.0)),
                BlockInfo::new(
                    true, 
                    false, 
                    0.1, 
                ),
                ActiveEvents::COLLISION_EVENTS,
                //Friction::coefficient(2.0),
            ));
    }
}
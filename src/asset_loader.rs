use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

//use crate::coll::DebugComp;

#[derive(Bundle)]
pub struct LoaderBundle {
    //tbh IDK if I need this
}

pub struct Asset_Loader_Plugin;

impl Plugin for Asset_Loader_Plugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SceneAsset>()
            //.init_resource::<LoadingAssets>()
            .add_systems(PreStartup, load_assets)
        ;
    }
}

#[derive(Resource, Debug, Default)]
pub struct SceneAsset {
    pub t_scorch: Handle<Image>,
    pub t_block: Handle<Image>,
}

fn load_assets(mut scene_assets: ResMut<SceneAsset>, asset_server: Res<AssetServer>) {
    let texture_handles: Vec<Handle<Image>> = vec![
        asset_server.load("sprites/t_scorch.png"),
        asset_server.load("sprites/t_block.png"),
    ];

    *scene_assets = SceneAsset {
        t_scorch: asset_server.load("sprites/t_scorch.png"),
        t_block: asset_server.load("sprites/t_block.png"),
    }
}

//I want to store relevent data within a struct elsewhere, however I want to have handles to be sent there here
// #[derive(Resource, Debug, Default)]
// struct LoadingAssets {
//     image_handles: Vec<Handle<Image>>,
// }

// fn preload_textures(
//     mut handle_res: ResMut<LoadingAssets>, 
//     asset_server: Res<AssetServer>
// ) {
    
//     handle_res.image_handles = texture_handles;
// }
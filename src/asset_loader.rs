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
            .add_systems(PreStartup, load_assets)
        ;
    }
}

#[derive(Resource, Debug, Default)]
pub struct SceneAsset {
    pub t_scorch: Handle<Image>,
}

fn load_assets(mut scene_assets: ResMut<SceneAsset>, asset_server: Res<AssetServer>) {
    *scene_assets = SceneAsset {
        t_scorch: asset_server.load("sprites/t_scorch.png"),
    }
}
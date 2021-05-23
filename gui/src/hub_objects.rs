use bevy::prelude::*;

pub enum HubAsset {
    TechnicMediumHub,
}

impl HubAsset {
    pub fn load(&self, asset_server: &Res<AssetServer>) -> Handle<Texture> {
        use HubAsset::*;

        let file = match self {
            TechnicMediumHub => "images/TechnicMediumHub.png",
        };

        asset_server.load(file)
    }
}

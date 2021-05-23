use crate::HubInfo;
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

pub fn draw_hubs(hubs_query: Query<(Entity, &HubInfo, &Sprite)>) {
    for (entity, hub, sprite) in hubs_query.iter() {
        println!(
            "Hub {} size is: {:?}, addr is {:?}",
            entity.id(),
            sprite.size,
            hub.addr
        );
    }
}

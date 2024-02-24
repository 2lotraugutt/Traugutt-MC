use valence::*;
use valence::prelude::*;

use crate::login::LoginEvent;

// use valence::command::handler::CommandResultEvent;
// use valence::command::scopes::CommandScopes;
// use valence::command_macros::Command;

#[derive(Debug, Resource)]
pub struct SpawnResource {
    pub layer_id: Option<Entity>,
}
impl SpawnResource {
    pub fn new() -> SpawnResource{
        SpawnResource{layer_id: None}
    }
}

pub struct SpawnPlugin;
impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        let login_asset = SpawnResource::new();
        app
            .insert_resource(login_asset)
            .add_systems(Startup, setup)
            .add_systems(Update, handle_login_event);
    }
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
    mut spawn_resource: ResMut<SpawnResource>
) {
    let mut login_layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);
    worldgen(&mut login_layer);

    let spawned = commands.spawn(login_layer);
    spawn_resource.layer_id = Some(spawned.id());
}

fn worldgen(
    layer: &mut LayerBundle,
) {
    for x in -10..10 {
        for z in -10..10 {
            layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }
    // let block = BlockState::BARRIER;
    let block = BlockState::STONE;
    for x in -10*10..10*10 {
        for z in -10*20..10*10 {
            layer.chunk.set_block([x, -1, z], block);
        }
    }
}

const SPAWN_POS: [f64; 3] = [
    0 as f64,
    0 as f64,
    0 as f64,
];
fn handle_login_event(
    mut events: EventReader<LoginEvent>,
    spawn_resource: ResMut<SpawnResource>,
    mut clients: Query<(
            &mut Client, // client
            &mut EntityLayerId, // layer_id
            &mut VisibleChunkLayer, // visable_chunk_layer
            &mut VisibleEntityLayers, // visable_chunk_layer
            &mut Position,
            &mut GameMode, // game_mode
        )>,
){
    for event in events.read() {
        let (
            ref mut client,
            ref mut layer_id,
            ref mut visible_chunk_layer,
            ref mut visible_entity_layer,
            ref mut pos,
            ref mut gamemode,
        ) = &mut clients.get_mut(event.player).unwrap();
        layer_id.0 = spawn_resource.layer_id.unwrap();
        visible_chunk_layer.0 = spawn_resource.layer_id.unwrap();
        visible_entity_layer.0.insert(spawn_resource.layer_id.unwrap());
        pos.set(SPAWN_POS);
        **gamemode = GameMode::Creative;
        client.send_chat_message(
            "This is the waiting room. ".into_text() +
            "Wait here".into_text().color(Color::BLUE).bold() +
            " to the begining of the competition".into_text()
        );
    }

}


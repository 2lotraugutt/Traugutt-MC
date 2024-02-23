use valence::*;
use valence::prelude::*;
use std::ops::DerefMut;

const SPAWN_POS: [f64; 3] = [
    0 as f64,
    128 as f64,
    0 as f64,
];

pub fn main () {
    App::new()
        .insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Offline,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
            ),
        )
        .run();
}

pub fn setup (
    mut commands: Commands,
    server: Res<Server>,
    mut dimensions: ResMut<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
    // mut command_scopes: ResMut<CommandScopeRegistry>,
) {
    dimensions
        .deref_mut()
        .insert(Ident::new("overworld").unwrap(), DimensionType::default());

    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);
    let mut login_layer = LayerBundle::new(ident!("nether"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    commands.spawn(layer);
    commands.spawn(login_layer);
}

fn init_clients(
    mut clients: Query<
        (
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut Position,
            &mut GameMode,
            // &mut Health,
            // &mut OpLevel,
            // &mut CommandScopes,
        ),
        Added<Client>,
    >,
    main_layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for (
        mut client,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut pos,
        mut game_mode,
        // mut health,
        // mut op_level,
        // mut permissions,
    ) in &mut clients
    {
        let layer = main_layers.iter().next().unwrap();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        // visible_entity_layers.0.insert(layer);
        pos.set(SPAWN_POS);
        *game_mode = GameMode::Spectator;
        // health.0 = 20.0;

        client.send_chat_message(
            "Welcome to ".into_text() +
            "2lo Traugutt".into_text().color(Color::GREEN).bold() +
            " Minecraft server".into_text()
        );
        // op_level.set(3);

        // permissions.add("all");
    }
}

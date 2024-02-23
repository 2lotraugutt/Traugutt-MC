use valence::*;
use valence::prelude::*;
use std::ops::DerefMut;

const SPAWN_POS: [f64; 3] = [
    1.5 as f64,
    128 as f64,
    1.5 as f64,
];

mod login;

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
    let mut login_layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    login::worldgen(&mut login_layer);
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
        mut health,
        mut permissions,
    ) in &mut clients
    {
        let layer = main_layers.iter().nth(1).unwrap();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        // visible_entity_layers.0.insert(layer);
        pos.set(SPAWN_POS);
        *game_mode = GameMode::Adventure;
        health.0 = 20.0;

        client.send_chat_message(
            "Welcome to ".into_text() +
            "2LO Traugutt".into_text().color(Color::GREEN).bold() +
            " Minecraft server".into_text()
        );
        client.send_chat_message(
            "Type ".into_text() +
            "/login password".into_text().color(Color::RED).bold() +
            " to login".into_text()
        );

        permissions.add("everyone");
        permissions.add("notloged");
    }
}

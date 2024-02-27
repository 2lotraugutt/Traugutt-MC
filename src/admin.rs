use valence::prelude::*;
use valence::*;

use command::handler::CommandResultEvent;
use command::parsers::entity_selector::{EntitySelector, EntitySelectors};
use command::{parsers, AddCommand};
use command_macros::Command;

use parsers::Vec3 as Vec3Parser;
use parsers::GreedyString;

use rand::prelude::IteratorRandom;
use valence::entity::living::LivingEntity;
// use valence::op_level::OpLevel;

pub struct AdminPlugin;
impl Plugin for AdminPlugin {
    fn build(&self, app: &mut App) {
        // let login_asset = LoginResource::serialize_from_file().unwrap();
        app
            .add_command::<TeleportCommand>()
            .add_command::<GamemodeCommand>()
            .add_command::<AnounceCommand>()
            .add_command::<PmCommand>()
            .add_systems(
                Update,
                (
                    handle_teleport_command,
                    handle_gamemode_command,
                    handle_anounce_command,
                    handle_pm_command,
                ),
            );
            // .insert_resource(login_asset)
            // .add_systems(Startup, setup)
            // .add_command::<LoginCommand>()
            // .add_event::<LoginEvent>()
            // .add_systems(Update, (handle_login_command,init_clients));
    }
}

#[derive(Command, Debug, Clone)]
#[paths("anounce {message}", "an {message}")]
#[scopes("mod.anounce", "admin.anounce")]
struct AnounceCommand {
     message: GreedyString
}

#[derive(Command, Debug, Clone)]
#[paths("pm {user} {message}")]
#[scopes("loged.pm")]
struct PmCommand {
     user: String,
     message: GreedyString
}

#[derive(Command, Debug, Clone)]
#[paths("tp")]
#[scopes("admin.teleport")]
enum TeleportCommand {
    #[paths = "{location}"]
    ExecutorToLocation { location: Vec3Parser },
    #[paths = "{target}"]
    ExecutorToTarget { target: EntitySelector },
    #[paths = "{from} {to}"]
    TargetToTarget {
        from: EntitySelector,
        to: EntitySelector,
    },
    #[paths = "{target} {location}"]
    TargetToLocation {
        target: EntitySelector,
        location: Vec3Parser,
    },
}

enum TeleportTarget {
    Targets(Vec<Entity>),
}

#[derive(Debug)]
enum TeleportDestination {
    Location(Vec3Parser),
    Target(Option<Entity>),
}

#[derive(Command, Debug, Clone)]
#[paths("gm")]
#[scopes("admin.gamemode")]
enum GamemodeCommand {
    #[paths("0 {target?}")]
    Survival { target: Option<EntitySelector> },
    #[paths("1 {target?}")]
    Creative { target: Option<EntitySelector> },
    #[paths("3 {target?}")]
    Adventure { target: Option<EntitySelector> },
    #[paths("2 {target?}")]
    Spectator { target: Option<EntitySelector> },
}

fn handle_gamemode_command(
    mut events: EventReader<CommandResultEvent<GamemodeCommand>>,
    mut clients: Query<(&mut Client, &mut GameMode, &Username, Entity)>,
    positions: Query<&Position>,
) {
    for event in events.read() {
        let game_mode_to_set = match &event.result {
            GamemodeCommand::Survival { .. } => GameMode::Survival,
            GamemodeCommand::Creative { .. } => GameMode::Creative,
            GamemodeCommand::Adventure { .. } => GameMode::Adventure,
            GamemodeCommand::Spectator { .. } => GameMode::Spectator,
        };

        let selector = match &event.result {
            GamemodeCommand::Survival { target } => target.clone(),
            GamemodeCommand::Creative { target } => target.clone(),
            GamemodeCommand::Adventure { target } => target.clone(),
            GamemodeCommand::Spectator { target } => target.clone(),
        };

        match selector {
            None => {
                let (_, mut game_mode, ..) = clients.get_mut(event.executor).unwrap();
                *game_mode = game_mode_to_set;
            }
            Some(selector) => match selector {
                EntitySelector::SimpleSelector(selector) => match selector {
                    EntitySelectors::AllEntities => {
                        for (_, mut game_mode, ..) in &mut clients.iter_mut() {
                            *game_mode = game_mode_to_set;
                        }
                    }
                    EntitySelectors::SinglePlayer(name) => {
                        let target = clients
                            .iter_mut()
                            .find(|(.., username, _)| username.0 == *name)
                            .map(|(.., target)| target);

                        match target {
                            None => {
                            }
                            Some(target) => {
                                let mut game_mode = clients.get_mut(target).unwrap().1;
                                *game_mode = game_mode_to_set;
                            }
                        }
                    }
                    EntitySelectors::AllPlayers => {
                        for (_ , mut game_mode, ..) in &mut clients.iter_mut() {
                            *game_mode = game_mode_to_set;
                        }
                    }
                    EntitySelectors::SelfPlayer => {
                        let (_, mut game_mode, ..) =
                            clients.get_mut(event.executor).unwrap();
                        *game_mode = game_mode_to_set;
                    }
                    EntitySelectors::NearestPlayer => {
                        let executor_pos = positions.get(event.executor).unwrap();
                        let target = clients
                            .iter_mut()
                            .filter(|(.., target)| *target != event.executor)
                            .min_by(|(.., target), (.., target2)| {
                                let target_pos = positions.get(*target).unwrap();
                                let target2_pos = positions.get(*target2).unwrap();
                                let target_dist = target_pos.distance(**executor_pos);
                                let target2_dist = target2_pos.distance(**executor_pos);
                                target_dist.partial_cmp(&target2_dist).unwrap()
                            })
                            .map(|(.., target)| target);

                        match target {
                            None => {
                            }
                            Some(target) => {
                                let mut game_mode = clients.get_mut(target).unwrap().1;
                                *game_mode = game_mode_to_set;

                            }
                        }
                    }
                    EntitySelectors::RandomPlayer => {
                        let target = clients
                            .iter_mut()
                            .choose(&mut rand::thread_rng())
                            .map(|(.., target)| target);

                        match target {
                            None => {
                            }
                            Some(target) => {
                                let mut game_mode = clients.get_mut(target).unwrap().1;
                                *game_mode = game_mode_to_set;

                            }
                        }
                    }
                },
                EntitySelector::ComplexSelector(_, _) => {
                }
            },
        }
    }
}

fn find_targets(
    living_entities: &Query<Entity, With<LivingEntity>>,
    clients: &mut Query<(Entity, &mut Client)>,
    positions: &Query<&mut Position>,
    entity_layers: &Query<&EntityLayerId>,
    usernames: &Query<(Entity, &Username)>,
    event: &CommandResultEvent<TeleportCommand>,
    target: &EntitySelector,
) -> Vec<Entity> {
    match target {
        EntitySelector::SimpleSelector(selector) => match selector {
            EntitySelectors::AllEntities => {
                let executor_entity_layer = *entity_layers.get(event.executor).unwrap();
                living_entities
                    .iter()
                    .filter(|entity| {
                        let entity_layer = entity_layers.get(*entity).unwrap();
                        entity_layer.0 == executor_entity_layer.0
                    })
                    .collect()
            }
            EntitySelectors::SinglePlayer(name) => {
                let target = usernames.iter().find(|(_, username)| username.0 == *name);
                match target {
                    None => {
                        vec![]
                    }
                    Some(target_entity) => {
                        vec![target_entity.0]
                    }
                }
            }
            EntitySelectors::AllPlayers => {
                let executor_entity_layer = *entity_layers.get(event.executor).unwrap();
                clients
                    .iter_mut()
                    .filter_map(|(entity, ..)| {
                        let entity_layer = entity_layers.get(entity).unwrap();
                        if entity_layer.0 == executor_entity_layer.0 {
                            Some(entity)
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            EntitySelectors::SelfPlayer => {
                vec![event.executor]
            }
            EntitySelectors::NearestPlayer => {
                let executor_entity_layer = *entity_layers.get(event.executor).unwrap();
                let executor_pos = positions.get(event.executor).unwrap();
                let target = clients
                    .iter_mut()
                    .filter(|(entity, ..)| {
                        *entity_layers.get(*entity).unwrap() == executor_entity_layer
                    })
                    .filter(|(target, ..)| *target != event.executor)
                    .map(|(target, ..)| target)
                    .min_by(|target, target2| {
                        let target_pos = positions.get(*target).unwrap();
                        let target2_pos = positions.get(*target2).unwrap();
                        let target_dist = target_pos.distance(**executor_pos);
                        let target2_dist = target2_pos.distance(**executor_pos);
                        target_dist.partial_cmp(&target2_dist).unwrap()
                    });
                match target {
                    None => {
                        vec![]
                    }
                    Some(target_entity) => {
                        vec![target_entity]
                    }
                }
            }
            EntitySelectors::RandomPlayer => {
                let executor_entity_layer = *entity_layers.get(event.executor).unwrap();
                let target = clients
                    .iter_mut()
                    .filter(|(entity, ..)| {
                        *entity_layers.get(*entity).unwrap() == executor_entity_layer
                    })
                    .choose(&mut rand::thread_rng())
                    .map(|(target, ..)| target);
                match target {
                    None => {
                        vec![]
                    }
                    Some(target_entity) => {
                        vec![target_entity]
                    }
                }
            }
        },
        EntitySelector::ComplexSelector(_, _) => {
            vec![]
        }
    }
}

fn handle_teleport_command(
    mut events: EventReader<CommandResultEvent<TeleportCommand>>,
    living_entities: Query<Entity, With<LivingEntity>>,
    mut clients: Query<(Entity, &mut Client)>,
    entity_layers: Query<&EntityLayerId>,
    mut positions: Query<&mut Position>,
    usernames: Query<(Entity, &Username)>,
) {
    for event in events.read() {
        let compiled_command = match &event.result {
            TeleportCommand::ExecutorToLocation { location } => (
                TeleportTarget::Targets(vec![event.executor]),
                TeleportDestination::Location(*location),
            ),
            TeleportCommand::ExecutorToTarget { target } => (
                TeleportTarget::Targets(vec![event.executor]),
                TeleportDestination::Target(
                    find_targets(
                        &living_entities,
                        &mut clients,
                        &positions,
                        &entity_layers,
                        &usernames,
                        event,
                        target,
                    )
                    .first()
                    .copied(),
                ),
            ),
            TeleportCommand::TargetToTarget { from, to } => (
                TeleportTarget::Targets(
                    find_targets(
                        &living_entities,
                        &mut clients,
                        &positions,
                        &entity_layers,
                        &usernames,
                        event,
                        from,
                    )
                    .to_vec(),
                ),
                TeleportDestination::Target(
                    find_targets(
                        &living_entities,
                        &mut clients,
                        &positions,
                        &entity_layers,
                        &usernames,
                        event,
                        to,
                    )
                    .first()
                    .copied(),
                ),
            ),
            TeleportCommand::TargetToLocation { target, location } => (
                TeleportTarget::Targets(
                    find_targets(
                        &living_entities,
                        &mut clients,
                        &positions,
                        &entity_layers,
                        &usernames,
                        event,
                        target,
                    )
                    .to_vec(),
                ),
                TeleportDestination::Location(*location),
            ),
        };

        let (TeleportTarget::Targets(targets), destination) = compiled_command;

        match destination {
            TeleportDestination::Location(location) => {
                for target in targets {
                    let mut pos = positions.get_mut(target).unwrap();
                    pos.0.x = location.x.get(pos.0.x as f32) as f64;
                    pos.0.y = location.y.get(pos.0.y as f32) as f64;
                    pos.0.z = location.z.get(pos.0.z as f32) as f64;
                }
            }
            TeleportDestination::Target(target) => {
                let target = target.unwrap();
                let target_pos = **positions.get(target).unwrap();
                for target in targets {
                    let mut position = positions.get_mut(target).unwrap();
                    position.0 = target_pos;
                }
            }
        }
    }
}

fn handle_anounce_command(
    mut events: EventReader<CommandResultEvent<AnounceCommand>>,
    mut clients: Query< &mut Client >,
) {
    for event in events.read() {
        let compiled_command = &event.result;
        for ref mut client in &mut clients {
            (*client).send_chat_message(
                "[Anouncment] ".into_text().color(Color::GOLD)+
                compiled_command.message.to_string().into_text().color(Color::WHITE)
                )
        }
    }
}

fn handle_pm_command(
    mut events: EventReader<CommandResultEvent<PmCommand>>,
    mut clients: Query<(&mut Client,  &Username)>,
    ){
        for event in events.read() {
        let org_user = clients.get(event.executor).unwrap().1.clone();
        let compiled_command = &event.result;
        for (ref mut client, ref username) in &mut clients {
            if username.to_string() == compiled_command.user {
                (*client).send_chat_message(
                    format!("{{{}}} ", org_user).into_text().color(Color::BLUE)+
                    compiled_command.message.to_string().into_text().color(Color::WHITE)
                    )
            }
        }
    }
}

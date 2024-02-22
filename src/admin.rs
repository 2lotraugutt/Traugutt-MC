use valence::*;
use command::handler::CommandResultEvent;
use command::parsers::entity_selector::{EntitySelector, EntitySelectors};
use command::parsers;
use command_macros::Command;
use parsers::Vec3 as Vec3Parser;
use rand::prelude::IteratorRandom;
use valence::entity::living::LivingEntity;
use valence::prelude::*;

#[derive(Command, Debug, Clone)]
#[paths("gamemode", "gm")]
#[scopes("valence.command.gamemode")]
pub enum GamemodeCommand {
    #[paths("survival {target?}", "{/} gms {target?}")]
    Survival { target: Option<EntitySelector> },
    #[paths("creative {target?}", "{/} gmc {target?}")]
    Creative { target: Option<EntitySelector> },
    #[paths("adventure {target?}", "{/} gma {target?}")]
    Adventure { target: Option<EntitySelector> },
    #[paths("spectator {target?}", "{/} gmspec {target?}")]
    Spectator { target: Option<EntitySelector> },
}

#[derive(Command, Debug, Clone)]
#[paths("teleport", "tp")]
#[scopes("valence.command.teleport")]
pub enum TeleportCommand {
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
                        let client = &mut clients.get_mut(event.executor).unwrap().1;
                        client.send_chat_message(format!("Could not find target: {}", name));
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
                        let mut client = clients.get_mut(event.executor).unwrap().1;
                        client.send_chat_message("Could not find target".to_string());
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
                        let mut client = clients.get_mut(event.executor).unwrap().1;
                        client.send_chat_message("Could not find target".to_string());
                        vec![]
                    }
                    Some(target_entity) => {
                        vec![target_entity]
                    }
                }
            }
        },
        EntitySelector::ComplexSelector(_, _) => {
            let mut client = clients.get_mut(event.executor).unwrap().1;
            client.send_chat_message("complex selector not implemented".to_string());
            vec![]
        }
    }
}
pub fn handle_teleport_command(
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

        println!(
            "executing teleport command {:#?} -> {:#?}",
            targets, destination
        );
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

pub fn handle_gamemode_command(
    mut events: EventReader<CommandResultEvent<GamemodeCommand>>,
    mut clients: Query<(&mut Client, &mut GameMode, &Username, Entity)>,
    positions: Query<&Position>,
) {
    for event in events.read() {
        println!("Gamemode");
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
                let (mut client, mut game_mode, ..) = clients.get_mut(event.executor).unwrap();
                *game_mode = game_mode_to_set;
                client.send_chat_message(format!(
                    "Gamemode command executor -> self executed with data:\n {:#?}",
                    &event.result
                ));
            }
            Some(selector) => match selector {
                EntitySelector::SimpleSelector(selector) => match selector {
                    EntitySelectors::AllEntities => {
                        for (mut client, mut game_mode, ..) in &mut clients.iter_mut() {
                            *game_mode = game_mode_to_set;
                            client.send_chat_message(format!(
                                "Gamemode command executor -> all entities executed with data:\n \
                                 {:#?}",
                                &event.result
                            ));
                        }
                    }
                    EntitySelectors::SinglePlayer(name) => {
                        let target = clients
                            .iter_mut()
                            .find(|(.., username, _)| username.0 == *name)
                            .map(|(.., target)| target);

                        match target {
                            None => {
                                let client = &mut clients.get_mut(event.executor).unwrap().0;
                                client
                                    .send_chat_message(format!("Could not find target: {}", name));
                            }
                            Some(target) => {
                                let mut game_mode = clients.get_mut(target).unwrap().1;
                                *game_mode = game_mode_to_set;

                                let client = &mut clients.get_mut(event.executor).unwrap().0;
                                client.send_chat_message(format!(
                                    "Gamemode command executor -> single player executed with \
                                     data:\n {:#?}",
                                    &event.result
                                ));
                            }
                        }
                    }
                    EntitySelectors::AllPlayers => {
                        for (mut client, mut game_mode, ..) in &mut clients.iter_mut() {
                            *game_mode = game_mode_to_set;
                            client.send_chat_message(format!(
                                "Gamemode command executor -> all entities executed with data:\n \
                                 {:#?}",
                                &event.result
                            ));
                        }
                    }
                    EntitySelectors::SelfPlayer => {
                        let (mut client, mut game_mode, ..) =
                            clients.get_mut(event.executor).unwrap();
                        *game_mode = game_mode_to_set;
                        client.send_chat_message(format!(
                            "Gamemode command executor -> self executed with data:\n {:#?}",
                            &event.result
                        ));
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
                                let client = &mut clients.get_mut(event.executor).unwrap().0;
                                client.send_chat_message("Could not find target".to_string());
                            }
                            Some(target) => {
                                let mut game_mode = clients.get_mut(target).unwrap().1;
                                *game_mode = game_mode_to_set;

                                let client = &mut clients.get_mut(event.executor).unwrap().0;
                                client.send_chat_message(format!(
                                    "Gamemode command executor -> single player executed with \
                                     data:\n {:#?}",
                                    &event.result
                                ));
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
                                let client = &mut clients.get_mut(event.executor).unwrap().0;
                                client.send_chat_message("Could not find target".to_string());
                            }
                            Some(target) => {
                                let mut game_mode = clients.get_mut(target).unwrap().1;
                                *game_mode = game_mode_to_set;

                                let client = &mut clients.get_mut(event.executor).unwrap().0;
                                client.send_chat_message(format!(
                                    "Gamemode command executor -> single player executed with \
                                     data:\n {:#?}",
                                    &event.result
                                ));
                            }
                        }
                    }
                },
                EntitySelector::ComplexSelector(_, _) => {
                    let client = &mut clients.get_mut(event.executor).unwrap().0;
                    client
                        .send_chat_message("Complex selectors are not implemented yet".to_string());
                }
            },
        }
    }
}

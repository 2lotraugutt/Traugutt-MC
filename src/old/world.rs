use command::CommandScopeRegistry;
use std::ops::DerefMut;
use std::collections::HashMap;
use command::scopes::CommandScopes;

use valence::*;
use command::AddCommand;

use bevy_ecs::query::WorldQuery;
use valence::entity::cow::CowEntityBundle;
use valence::entity::entity::Flags;
use valence::entity::living::Health;
use valence::entity::pig::PigEntityBundle;
use valence::entity::player::PlayerEntityBundle;
use valence::entity::{EntityAnimations, EntityStatuses, OnGround, Velocity};
use valence::interact_block::InteractBlockEvent;
use valence::inventory::HeldItem;
use valence::log::debug;
use valence::math::Vec3Swizzles;
use valence::nbt::{compound, List};
use valence::prelude::*;
use valence::scoreboard::*;
use valence::status::RequestRespawnEvent;
use valence::op_level::OpLevel;

const ARENA_Y: i32 = 64;
const ARENA_MID_WIDTH: i32 = 2;
const SPAWN_BOX: [i32; 3] = [0, ARENA_Y + 20, 0];
const SPAWN_POS: [f64; 3] = [
    SPAWN_BOX[0] as f64,
    SPAWN_BOX[1] as f64 + 1.0,
    SPAWN_BOX[2] as f64,
];
const SPAWN_BOX_WIDTH: i32 = 5;
const SPAWN_BOX_HEIGHT: i32 = 4;


#[derive(Debug, Resource)]
struct CtfGlobals {
    pub scoreboard_layer: Entity,

    pub red_flag: BlockPos,
    pub blue_flag: BlockPos,

    pub red_capture_trigger: TriggerArea,
    pub blue_capture_trigger: TriggerArea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TriggerArea {
    pub a: BlockPos,
    pub b: BlockPos,
}

impl TriggerArea {
    pub fn new(a: impl Into<BlockPos>, b: impl Into<BlockPos>) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
        }
    }

    pub fn contains(&self, pos: BlockPos) -> bool {
        let min = BlockPos::new(
            self.a.x.min(self.b.x),
            self.a.y.min(self.b.y),
            self.a.z.min(self.b.z),
        );
        let max = BlockPos::new(
            self.a.x.max(self.b.x),
            self.a.y.max(self.b.y),
            self.a.z.max(self.b.z),
        );

        pos.x >= min.x
            && pos.x <= max.x
            && pos.y >= min.y
            && pos.y <= max.y
            && pos.z >= min.z
            && pos.z <= max.z
    }

    pub fn contains_pos(&self, pos: DVec3) -> bool {
        self.contains(pos.into())
    }

    pub fn iter_block_pos(&self) -> impl Iterator<Item = BlockPos> {
        let min = BlockPos::new(
            self.a.x.min(self.b.x),
            self.a.y.min(self.b.y),
            self.a.z.min(self.b.z),
        );
        let max = BlockPos::new(
            self.a.x.max(self.b.x),
            self.a.y.max(self.b.y),
            self.a.z.max(self.b.z),
        );

        (min.x..=max.x)
            .flat_map(move |x| (min.y..=max.y).map(move |y| (x, y)))
            .flat_map(move |(x, y)| (min.z..=max.z).map(move |z| BlockPos::new(x, y, z)))
    }
}

#[derive(Debug, Default, Resource)]
struct Score {
    pub scores: HashMap<Team, u32>,
}

impl Score {
    pub fn render_scores(&self) -> Text {
        let mut text = "Scores:\n".into_text();
        for team in Team::iter() {
            let score = self.scores.get(&team).unwrap_or(&0);
            text += team.team_text() + ": " + score.to_string() + "\n";
        }
        text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
enum Team {
    Red,
    Blue,
}

impl Team {
    pub fn spawn_pos(&self) -> DVec3 {
        [
            match self {
                Team::Red => -40.0,
                Team::Blue => 40.0,
            },
            ARENA_Y as f64 + 1.0,
            0.0,
        ]
        .into()
    }

    pub fn team_text(&self) -> Text {
        match self {
            Team::Red => "RED".color(Color::RED).bold(),
            Team::Blue => "BLUE".color(Color::BLUE).bold(),
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Team::Red, Team::Blue].iter().copied()
    }
}

pub fn worldgen(
    layer: &mut LayerBundle,
    commands: &mut Commands,
    ) {
    for z in -5..5 {
        for x in -5..5 {
            layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    for z in -50..50 {
        for x in -50..50 {
            let block = match x {
                x if x < -ARENA_MID_WIDTH => BlockState::RED_CONCRETE,
                x if x > ARENA_MID_WIDTH => BlockState::BLUE_CONCRETE,
                _ => BlockState::WHITE_CONCRETE,
            };
            layer.chunk.set_block([x, ARENA_Y, z], block);
        }
    }

    let red_flag = build_flag(
        &mut layer,
        Team::Red,
        BlockPos {
            x: -48,
            y: ARENA_Y + 1,
            z: 0,
        },
    );
    let blue_flag = build_flag(
        &mut layer,
        Team::Blue,
        BlockPos {
            x: 48,
            y: ARENA_Y + 1,
            z: 0,
        },
    );


    let ctf_objective_layer = commands.spawn(EntityLayer::new(&server)).id();
    let ctf_objective = ObjectiveBundle {
        name: Objective::new("ctf-captures"),
        display: ObjectiveDisplay("Captures".into_text()),
        layer: EntityLayerId(ctf_objective_layer),
        ..Default::default()
    };
    commands.spawn(ctf_objective);

    let red_capture_trigger =
        TriggerArea::new(red_flag.offset(-5, -3, -5), red_flag.offset(5, 3, 5));
    let blue_capture_trigger =
        TriggerArea::new(blue_flag.offset(-5, -3, -5), blue_flag.offset(5, 3, 5));
    let mappos = CtfGlobals {
        scoreboard_layer: ctf_objective_layer,

        red_flag,
        blue_flag,

        red_capture_trigger,
        blue_capture_trigger,
    };

    commands.insert_resource(mappos);
    commands.insert_resource(FlagManager {
        red: None,
        blue: None,
    });

    let ctf_team_layers = CtfLayers::init(&mut commands, &server);

    // add some debug entities to the ctf entity layers
    let mut flags = Flags::default();
    flags.set_glowing(true);
    let mut pig = commands.spawn(PigEntityBundle {
        layer: EntityLayerId(ctf_team_layers.friendly_layers[&Team::Red]),
        position: Position([-30.0, 65.0, 2.0].into()),
        entity_flags: flags.clone(),
        ..Default::default()
    });
    pig.insert(Team::Red);

    let mut cow = commands.spawn(CowEntityBundle {
        layer: EntityLayerId(ctf_team_layers.friendly_layers[&Team::Blue]),
        position: Position([30.0, 65.0, 2.0].into()),
        entity_flags: flags,
        ..Default::default()
    });
    cow.insert(Team::Blue);

    commands.insert_resource(ctf_team_layers);
    commands.insert_resource(Score::default());
}

fn build_flag(layer: &mut LayerBundle, team: Team, pos: impl Into<BlockPos>) -> BlockPos {
    let mut pos = pos.into();

    // build the flag pole
    for _ in 0..3 {
        layer.chunk.set_block(pos, BlockState::OAK_FENCE);
        pos.y += 1;
    }
    let moving_east = pos.x < 0;
    layer.chunk.set_block(
        pos,
        BlockState::OAK_FENCE.set(
            if moving_east {
                PropName::East
            } else {
                PropName::West
            },
            PropValue::True,
        ),
    );
    pos.x += if pos.x < 0 { 1 } else { -1 };
    layer.chunk.set_block(
        pos,
        BlockState::OAK_FENCE
            .set(PropName::East, PropValue::True)
            .set(PropName::West, PropValue::True),
    );
    pos.x += if pos.x < 0 { 1 } else { -1 };
    layer.chunk.set_block(
        pos,
        BlockState::OAK_FENCE.set(
            if moving_east {
                PropName::West
            } else {
                PropName::East
            },
            PropValue::True,
        ),
    );
    pos.y -= 1;

    // build the flag
    layer.chunk.set_block(
        pos,
        match team {
            Team::Red => BlockState::RED_WOOL,
            Team::Blue => BlockState::BLUE_WOOL,
        },
    );

    pos
}

pub fn build_spawn_box(layer: &mut LayerBundle, pos: impl Into<BlockPos>, commands: &mut Commands) {
    let pos = pos.into();

    let spawn_box_block = BlockState::GLASS;

    // build floor and roof
    for z in -SPAWN_BOX_WIDTH..=SPAWN_BOX_WIDTH {
        for x in -SPAWN_BOX_WIDTH..=SPAWN_BOX_WIDTH {
            layer
                .chunk
                .set_block([pos.x + x, pos.y, pos.z + z], spawn_box_block);
            layer.chunk.set_block(
                [pos.x + x, pos.y + SPAWN_BOX_HEIGHT, pos.z + z],
                spawn_box_block,
            );
        }
    }

    // build walls
    for z in [-SPAWN_BOX_WIDTH, SPAWN_BOX_WIDTH] {
        for x in -SPAWN_BOX_WIDTH..=SPAWN_BOX_WIDTH {
            for y in pos.y..=pos.y + SPAWN_BOX_HEIGHT - 1 {
                layer
                    .chunk
                    .set_block([pos.x + x, y, pos.z + z], spawn_box_block);
            }
        }
    }

    for x in [-SPAWN_BOX_WIDTH, SPAWN_BOX_WIDTH] {
        for z in -SPAWN_BOX_WIDTH..=SPAWN_BOX_WIDTH {
            for y in pos.y..=pos.y + SPAWN_BOX_HEIGHT - 1 {
                layer
                    .chunk
                    .set_block([pos.x + x, y, pos.z + z], spawn_box_block);
            }
        }
    }

    // build team selector portals
    for (block, offset) in [
        (
            BlockState::RED_CONCRETE,
            BlockPos::new(-SPAWN_BOX_WIDTH, 0, SPAWN_BOX_WIDTH - 2),
        ),
        (
            BlockState::BLUE_CONCRETE,
            BlockPos::new(SPAWN_BOX_WIDTH - 2, 0, SPAWN_BOX_WIDTH - 2),
        ),
    ] {
        for z in 0..3 {
            for x in 0..3 {
                layer.chunk.set_block(
                    [pos.x + offset.x + x, pos.y + offset.y, pos.z + offset.z + z],
                    block,
                );
            }
        }
    }

    let red = [
        pos.x - SPAWN_BOX_WIDTH + 1,
        pos.y,
        pos.z + SPAWN_BOX_WIDTH - 1,
    ];
    let red_area = TriggerArea::new(red, red);
    let blue = [
        pos.x + SPAWN_BOX_WIDTH - 1,
        pos.y,
        pos.z + SPAWN_BOX_WIDTH - 1,
    ];
    let blue_area = TriggerArea::new(blue, blue);
    let portals = Portals {
        portals: HashMap::from_iter(vec![(Team::Red, red_area), (Team::Blue, blue_area)]),
    };

    for area in portals.portals.values() {
        for pos in area.iter_block_pos() {
            layer.chunk.set_block(pos, BlockState::AIR);
        }
        layer
            .chunk
            .set_block(area.a.offset(0, -1, 0), BlockState::BARRIER);
    }

    commands.insert_resource(portals);

    // build instruction signs

    let sign_pos = pos.offset(0, 2, SPAWN_BOX_WIDTH - 1);
    layer.chunk.set_block(
        sign_pos,
        Block {
            state: BlockState::OAK_WALL_SIGN.set(PropName::Rotation, PropValue::_3),
            nbt: Some(compound! {
                "front_text" => compound! {
                    "messages" => List::String(vec![
                        "Capture".color(Color::YELLOW).bold().to_string(),
                        "the".color(Color::YELLOW).bold().to_string(),
                        "Flag!".color(Color::YELLOW).bold().to_string(),
                        "Select a Team".color(Color::WHITE).italic().to_string(),
                    ])
                },
            }),
        },
    );

    layer.chunk.set_block(
        sign_pos.offset(-1, 0, 0),
        Block {
            state: BlockState::OAK_WALL_SIGN.set(PropName::Rotation, PropValue::_3),
            nbt: Some(compound! {
                "front_text" => compound! {
                    "messages" => List::String(vec![
                        "".into_text().to_string(),
                        ("Join ".bold().color(Color::WHITE) + Team::Red.team_text()).to_string(),
                        "=>".bold().color(Color::WHITE).to_string(),
                        "".into_text().to_string(),
                    ])
                },
            }),
        },
    );

    layer.chunk.set_block(
        sign_pos.offset(1, 0, 0),
        Block {
            state: BlockState::OAK_WALL_SIGN.set(PropName::Rotation, PropValue::_3),
            nbt: Some(compound! {
                "front_text" => compound! {
                    "messages" => List::String(vec![
                        "".into_text().to_string(),
                        ("Join ".bold().color(Color::WHITE) + Team::Blue.team_text()).to_string(),
                        "<=".bold().color(Color::WHITE).to_string(),
                        "".into_text().to_string(),
                    ])
                },
            }),
        },
    );
}

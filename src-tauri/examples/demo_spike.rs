use cs2_demoparser::first_pass::parser_settings::ParserInputs;
use cs2_demoparser::parse_demo::{Parser, ParsingMode};
use cs2_demoparser::second_pass::parser_settings::create_huffman_lookup_table;
use cs2_demoparser::second_pass::variants::VarVec;

fn main() {
    let path = std::env::args().nth(1).expect("demo path");
    let bytes = std::fs::read(&path).expect("read demo");
    let huffman = create_huffman_lookup_table();
    let wanted_events = [
        "round_start",
        "round_freeze_end",
        "round_end",
        "round_officially_ended",
        "player_death",
        "player_hurt",
        "bomb_beginplant",
        "bomb_planted",
        "bomb_begindefuse",
        "bomb_defused",
        "bomb_exploded",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let player_props = [
        ("health", "CCSPlayerPawn.m_iHealth"),
        ("life_state", "CCSPlayerPawn.m_lifeState"),
        ("armor_value", "CCSPlayerPawn.m_ArmorValue"),
        ("team_num", "CCSPlayerPawn.m_iTeamNum"),
        (
            "current_equip_value",
            "CCSPlayerPawn.m_unCurrentEquipmentValue",
        ),
        ("active_weapon_name", "weapon_name"),
        ("inventory", "inventory"),
    ];
    let settings = ParserInputs {
        real_name_to_og_name: player_props
            .iter()
            .map(|(friendly, real)| (real.to_string(), friendly.to_string()))
            .collect(),
        wanted_players: vec![],
        wanted_player_props: player_props
            .iter()
            .map(|(_, real)| real.to_string())
            .collect(),
        wanted_other_props: vec![],
        wanted_prop_states: Default::default(),
        wanted_ticks: vec![],
        wanted_events,
        parse_ents: true,
        parse_projectiles: false,
        parse_grenades: false,
        only_header: false,
        only_convars: false,
        huffman_lookup_table: &huffman,
        order_by_steamid: false,
        list_props: false,
        fallback_bytes: None,
    };
    let started = std::time::Instant::now();
    let output = Parser::new(settings, ParsingMode::ForceSingleThreaded)
        .parse_demo(&bytes)
        .expect("parse demo");
    let mut counts = std::collections::BTreeMap::new();
    for event in &output.game_events {
        *counts.entry(event.name.as_str()).or_insert(0usize) += 1;
    }
    println!("elapsed_ms={}", started.elapsed().as_millis());
    let header = output
        .header
        .unwrap_or_default()
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    println!("header={}", serde_json::to_string(&header).unwrap());
    println!("players={}", serde_json::to_string(&output.roster).unwrap());
    println!("event_counts={}", serde_json::to_string(&counts).unwrap());
    for prop in &output.prop_controller.prop_infos {
        if player_props
            .iter()
            .any(|(friendly, _)| *friendly == prop.prop_friendly_name)
        {
            let populated = output
                .df_per_player
                .values()
                .filter(|columns| {
                    columns
                        .get(&prop.id)
                        .and_then(|column| column.data.as_ref())
                        .is_some_and(|data| match data {
                            VarVec::Bool(values) => values.iter().any(Option::is_some),
                            VarVec::I32(values) => values.iter().any(Option::is_some),
                            VarVec::F32(values) => values.iter().any(Option::is_some),
                            VarVec::String(values) => values.iter().any(Option::is_some),
                            VarVec::U32(values) => values.iter().any(Option::is_some),
                            VarVec::U64(values) => values.iter().any(Option::is_some),
                            VarVec::StringVec(values) => !values.is_empty(),
                            VarVec::U64Vec(values) => !values.is_empty(),
                            VarVec::U32Vec(values) => !values.is_empty(),
                            VarVec::XYVec(values) => values.iter().any(Option::is_some),
                            VarVec::XYZVec(values) => values.iter().any(Option::is_some),
                            VarVec::Stickers(values) => !values.is_empty(),
                            VarVec::InputHistory(values) => !values.is_empty(),
                        })
                })
                .count();
            println!(
                "prop={} id={} populated_players={}",
                prop.prop_friendly_name, prop.id, populated
            );
        }
    }
    for event in output.game_events.iter().take(8) {
        println!("event={}", serde_json::to_string(event).unwrap());
    }
}

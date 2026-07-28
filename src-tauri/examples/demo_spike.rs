use cs2_demoparser::first_pass::parser_settings::ParserInputs;
use cs2_demoparser::parse_demo::{Parser, ParsingMode};
use cs2_demoparser::second_pass::parser_settings::create_huffman_lookup_table;

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
    let settings = ParserInputs {
        real_name_to_og_name: Default::default(),
        wanted_players: vec![],
        wanted_player_props: vec![],
        wanted_other_props: vec![],
        wanted_prop_states: Default::default(),
        wanted_ticks: vec![],
        wanted_events,
        parse_ents: false,
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
    for event in output.game_events.iter().take(8) {
        println!("event={}", serde_json::to_string(event).unwrap());
    }
}

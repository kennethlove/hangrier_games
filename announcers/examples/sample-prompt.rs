//! Generate and print commentary using a locally-running Ollama instance.
//! Run with: cargo run --package announcers --features ollama --example sample-prompt 2>/dev/null

#[tokio::main]
async fn main() {
    use announcers::*;

    // Simulate mid-game state: Day 5, 12 tributes left, Cato on a streak.
    let header = GameStateSnapshot {
        day: 5,
        phase: "day".into(),
        alive_count: 12,
        kill_leaders: vec![KillLeader {
            name: "Cato".into(),
            district: 2,
            kill_count: 2,
        }],
        alliances: vec![],
        hot_zones: vec![AreaActivity {
            name: "Cornucopia".into(),
            activity_level: "hot".into(),
        }],
        killing_sprees: vec![KillingSpree {
            name: "Cato".into(),
            district: 2,
            streak: 4,
            label: "on fire".into(),
        }],
    };

    let events = vec![
        shared::messages::GameMessage {
            identifier: "evt-1".into(),
            source: shared::messages::MessageSource::Tribute("id-Cato".into()),
            game_day: 5,
            phase: shared::messages::Phase::Day,
            tick: 1,
            emit_index: 1,
            subject: String::new(),
            timestamp: chrono::Utc::now(),
            content: "Cato attacks Peeta, and wins decisively!".into(),
            payload: shared::messages::MessagePayload::CombatSwing(
                shared::combat_beat::CombatBeat {
                    attacker: shared::messages::TributeRef {
                        identifier: "id-Cato".into(),
                        name: "Cato".into(),
                    },
                    target: shared::messages::TributeRef {
                        identifier: "id-Peeta".into(),
                        name: "Peeta".into(),
                    },
                    weapon: None,
                    shield: None,
                    wear: vec![],
                    outcome: shared::combat_beat::SwingOutcome::Wound { damage: 7 },
                    stress: Default::default(),
                    attacker_stamina_cost: 3,
                    target_stamina_cost: 2,
                },
            ),
        },
        shared::messages::GameMessage {
            identifier: "evt-2".into(),
            source: shared::messages::MessageSource::Tribute("id-Cato".into()),
            game_day: 5,
            phase: shared::messages::Phase::Day,
            tick: 2,
            emit_index: 2,
            subject: String::new(),
            timestamp: chrono::Utc::now(),
            content: "Cato kills Peeta!".into(),
            payload: shared::messages::MessagePayload::TributeKilled {
                victim: shared::messages::TributeRef {
                    identifier: "id-Peeta".into(),
                    name: "Peeta".into(),
                },
                killer: Some(shared::messages::TributeRef {
                    identifier: "id-Cato".into(),
                    name: "Cato".into(),
                }),
                cause: shared::afflictions::DeathCause::Combat,
            },
        },
        shared::messages::GameMessage {
            identifier: "evt-3".into(),
            source: shared::messages::MessageSource::Tribute("id-Katniss".into()),
            game_day: 5,
            phase: shared::messages::Phase::Day,
            tick: 3,
            emit_index: 3,
            subject: String::new(),
            timestamp: chrono::Utc::now(),
            content: "Katniss moves from Forest to Cornucopia.".into(),
            payload: shared::messages::MessagePayload::TributeMoved {
                tribute: shared::messages::TributeRef {
                    identifier: "id-Katniss".into(),
                    name: "Katniss".into(),
                },
                from: shared::messages::AreaRef {
                    identifier: "Forest".into(),
                    name: "Forest".into(),
                },
                to: shared::messages::AreaRef {
                    identifier: "Cornucopia".into(),
                    name: "Cornucopia".into(),
                },
            },
        },
    ];

    let histories = vec![
        TributeDigest {
            identifier: "id-Cato".into(),
            name: "Cato".into(),
            district: 2,
            status: "alive".into(),
            injury_level: "unharmed".into(),
            location: "Cornucopia".into(),
            allies: vec![],
            kill_streak: 4,
            notable_events: vec![
                "Cato is on fire — 4 kills in a row!".into(),
                "Killed Peeta (combat)".into(),
                "Killed Clove (combat)".into(),
                "Killed Marvel (combat)".into(),
                "Killed Rue (combat)".into(),
            ],
            highlights: vec![
                "Killed Rue (combat)".into(),
                "Killed Marvel (combat)".into(),
                "Killed Clove (combat)".into(),
                "Killed Peeta (combat)".into(),
            ],
        },
        TributeDigest {
            identifier: "id-Katniss".into(),
            name: "Katniss".into(),
            district: 12,
            status: "alive".into(),
            injury_level: "wounded".into(),
            location: "Forest".into(),
            allies: vec![],
            kill_streak: 1,
            notable_events: vec!["Found bow in Forest".into()],
            highlights: vec!["Killed Marvel (combat)".into()],
        },
        TributeDigest {
            identifier: "id-Peeta".into(),
            name: "Peeta".into(),
            district: 12,
            status: "deceased".into(),
            injury_level: "deceased".into(),
            location: "Cornucopia".into(),
            allies: vec![],
            kill_streak: 0,
            notable_events: vec!["Killed by Cato".into()],
            highlights: vec!["Killed by Cato (combat)".into()],
        },
    ];

    let commentator = OllamaCommentator::new();
    let package = BroadcastPackageBuilder::build(header, &events, histories);

    println!("=== PROMPT SENT TO OLLAMA ===\n");
    println!("{}", commentator.build_prompt(&package));
    println!("\n=== GENERATING COMMENTARY... ===\n");

    match commentator.generate(&package).await {
        Ok(segment) => {
            println!("=== COMMENTARY OUTPUT ===\n");
            for line in &segment.lines {
                println!("[{}] {}", line.speaker, line.text);
            }
            println!(
                "\n({} lines, model: {})",
                segment.lines.len(),
                segment.model_used
            );
        }
        Err(e) => {
            eprintln!("Error generating commentary: {e}");
        }
    }
}

use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload, TributeRef};

#[derive(Props, PartialEq, Clone)]
pub struct AllianceCardProps {
    pub message: GameMessage,
}

#[component]
pub fn AllianceCard(props: AllianceCardProps) -> Element {
    let (icon, body) = match &props.message.payload {
        MessagePayload::AllianceFormed { members } => {
            ("🤝", format!("{} formed an alliance", joined(members)))
        }
        MessagePayload::AllianceProposed { proposer, target } => (
            "📜",
            format!("{} proposed an alliance to {}", proposer.name, target.name),
        ),
        MessagePayload::AllianceDissolved { members, reason } => {
            ("💔", format!("{} dissolved ({reason})", joined(members)))
        }
        MessagePayload::BetrayalTriggered { betrayer, victim } => {
            ("🗡️", format!("{} betrayed {}", betrayer.name, victim.name))
        }
        MessagePayload::TrustShockBreak { tribute, partner } => (
            "⚡",
            format!("{} broke trust with {}", tribute.name, partner.name),
        ),
        _ => ("🤝", "alliance event".to_string()),
    };
    rsx! {
        article { class: "rounded border-l-4 border-emerald-500 bg-emerald-50  p-3",
            header { class: "font-semibold", "{icon} {body}" }
        }
    }
}

fn joined(members: &[TributeRef]) -> String {
    members
        .iter()
        .map(|m| m.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

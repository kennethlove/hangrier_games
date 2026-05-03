use crate::cache::QueryError;
use crate::components::game_tributes::GameTributesQ;
use crate::components::icons::uturn::UTurnIcon;
use crate::components::info_detail::InfoDetail;
use crate::components::item_icon::ItemIcon;
use crate::components::tribute_status_icon::TributeStatusIcon;
use crate::components::tribute_survival_section::TributeSurvivalSection;
use crate::env::APP_API_HOST;
use crate::http::WithCredentials;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::messages::GameMessage;
use game::tributes::statuses::TributeStatus;
use game::tributes::traits::Trait;
use game::tributes::{Attributes, Tribute};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TributeQ;

impl QueryCapability for TributeQ {
    type Ok = Box<Tribute>;
    type Err = QueryError;
    type Keys = (String, String);

    async fn run(&self, keys: &(String, String)) -> Result<Box<Tribute>, QueryError> {
        let (game_identifier, tribute_identifier) = keys;
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                format!(
                    "{}/api/games/{}/tributes/{}",
                    APP_API_HOST, game_identifier, tribute_identifier
                ),
            )
            .with_credentials();
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Option<Tribute>>().await {
                        Ok(Some(tribute)) => Ok(Box::new(tribute)),
                        _ => Err(QueryError::TributeNotFound(tribute_identifier.to_string())),
                    }
                } else {
                    Err(QueryError::TributeNotFound(tribute_identifier.to_string()))
                }
            }
            Err(_) => Err(QueryError::Unknown),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TributeLogQ;

impl QueryCapability for TributeLogQ {
    type Ok = Vec<GameMessage>;
    type Err = QueryError;
    type Keys = (String, String);

    async fn run(&self, keys: &(String, String)) -> Result<Vec<GameMessage>, QueryError> {
        let (game_identifier, identifier) = keys;
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                format!(
                    "{}/api/games/{}/tributes/{}/log",
                    APP_API_HOST, game_identifier, identifier
                ),
            )
            .with_credentials();
        match request.send().await {
            Ok(response) => match response.json::<Vec<GameMessage>>().await {
                Ok(logs) => Ok(logs),
                Err(_) => Err(QueryError::TributeNotFound(identifier.to_string())),
            },
            Err(_) => Err(QueryError::TributeNotFound(identifier.to_string())),
        }
    }
}

/// Tailwind utility classes for a trait chip. Mapping is semantic: combat
/// stance traits get warm/cool tones, social traits map to trust/danger
/// signals, mental/physical traits get neutral-ish accents. Keep contrast
/// readable across all three themes (dark amber, dark green, light stone).
pub(crate) fn trait_chip_classes(t: &Trait) -> &'static str {
    match t {
        // Social: trust signals
        Trait::Loyal => "bg-green-700/40 text-green-100 border border-green-500/60",
        Trait::Friendly => "bg-emerald-700/40 text-emerald-100 border border-emerald-500/60",
        Trait::Treacherous => "bg-red-700/40 text-red-100 border border-red-500/60",
        Trait::Paranoid => "bg-yellow-700/40 text-yellow-100 border border-yellow-500/60",
        Trait::LoneWolf => "bg-slate-700/40 text-slate-100 border border-slate-400/60",
        // Combat stance
        Trait::Aggressive => "bg-orange-700/40 text-orange-100 border border-orange-500/60",
        Trait::Reckless => "bg-rose-700/40 text-rose-100 border border-rose-500/60",
        Trait::Defensive => "bg-sky-700/40 text-sky-100 border border-sky-500/60",
        Trait::Cautious => "bg-cyan-700/40 text-cyan-100 border border-cyan-500/60",
        // Mental
        Trait::Cunning => "bg-purple-700/40 text-purple-100 border border-purple-500/60",
        Trait::Dim => "bg-stone-700/40 text-stone-100 border border-stone-500/60",
        Trait::Resilient => "bg-teal-700/40 text-teal-100 border border-teal-500/60",
        Trait::Fragile => "bg-pink-700/40 text-pink-100 border border-pink-500/60",
        // Physical
        Trait::Tough => "bg-amber-700/40 text-amber-100 border border-amber-500/60",
        Trait::Asthmatic => "bg-indigo-700/40 text-indigo-100 border border-indigo-500/60",
        Trait::Nearsighted => "bg-fuchsia-700/40 text-fuchsia-100 border border-fuchsia-500/60",
        // Survival
        Trait::Builder => "bg-yellow-700/40 text-yellow-100 border border-yellow-500/60",
        Trait::ResourcefulForager => "bg-lime-700/40 text-lime-100 border border-lime-500/60",
    }
}

#[component]
pub fn TributeDetail(game_identifier: String, tribute_identifier: String) -> Element {
    let tribute_query = use_query(Query::new(
        (game_identifier.clone(), tribute_identifier.clone()),
        TributeQ,
    ));
    let reader = tribute_query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled {
            res: Ok(tribute), ..
        } => {
            rsx! {
                div {
                    class: "flex flex-row gap-4 mb-4 place-items-center place-content-between",
                    h2 {
                        class: r#"
                        theme1:text-2xl
                        theme1:font-[Cinzel]
                        theme1:text-amber-300

                        theme2:font-[Playfair_Display]
                        theme2:text-3xl
                        theme2:text-green-200

                        theme3:font-[Orbitron]
                        theme3:text-2xl
                        theme3:text-stone-700
                        "#,

                        "{tribute.name}"
                    }

                    span {
                        class: "pr-4 sm:pr-0",
                        Link {
                            to: Routes::GamePage {
                                identifier: game_identifier.clone()
                            },
                            UTurnIcon {
                                class: r#"
                                size-5
                                theme1:fill-amber-500
                                theme1:hover:fill-amber-200

                                theme2:fill-green-200/50
                                theme2:hover:fill-green-200

                                theme3:fill-amber-600/50
                                theme3:hover:fill-amber-600
                                "#,
                            }
                        }
                    }
                }

                div {
                    class: r#"
                    pr-4
                    sm:pr-0
                    grid
                    gap-2
                    grid-cols-none
                    sm:grid-cols-2
                    sm:gap-4
                    lg:grid-cols-3
                    lg:gap-8
                    xl:grid-cols-4

                    theme1:text-stone-200
                    theme2:text-green-200
                    "#,

                    InfoDetail {
                        title: "Overview",
                        open: true,
                        img {
                            class: "mb-4",
                            src: "{tribute.avatar()}",
                        }
                        dl {
                            class: "grid grid-cols-2 gap-4",
                            dt { "District" }
                            dd { "{tribute.district}" }
                            dt { "Location" }
                            dd { "{tribute.area}" }
                            dt { "Status" }
                            dd {
                                class: "flex flex-row gap-2 flex-wrap",
                                TributeStatusIcon {
                                    status: tribute.status.clone(),
                                    css_class: r#"
                                    size-5
                                    theme1:fill-stone-200
                                    theme2:fill-green-200
                                    "#
                                }
                                span {
                                    class: "text-sm",
                                    "{tribute.status.to_string()}"
                                }
                            }
                            dt { "Outlook" }
                            dd { "TODO" }
                        }
                    }

                    TributeSurvivalSection {
                        tribute: (**tribute).clone(),
                        current_phase: None,
                    }

                    InfoDetail {
                        title: "Inventory",
                        open: false,
                        ul {
                            class: "grid grid-cols-3 auto-cols-auto grid-flow-row gap-2",
                            for item in tribute.clone().items {
                                li {
                                    class: "flex flex-row flex-wrap gap-2 items-center",
                                    ItemIcon {
                                        item: item.clone(),
                                        css_class: r#"
                                        size-8
                                        theme1:fill-amber-500
                                        theme2:fill-green-200
                                        "#,
                                    }
                                    span {
                                        class: "text-sm",
                                        "{item.to_string()}"
                                    }
                                }
                            }
                        }
                    }

                    InfoDetail {
                        title: "Attributes",
                        open: false,
                        TributeAttributes { attributes: tribute.attributes.clone() }
                    }

                    if !tribute.traits.is_empty() {
                        InfoDetail {
                            title: "Traits",
                            open: false,
                            TributeTraits {
                                traits: tribute.traits.clone(),
                                turns_since_last_betrayal: tribute.turns_since_last_betrayal,
                            }
                        }
                    }

                    InfoDetail {
                        title: "Allies",
                        open: false,
                        TributeAllies {
                            game_identifier: game_identifier.clone(),
                            ally_ids: tribute.allies.clone(),
                        }
                    }

                    if !tribute.clone().editable {
                        InfoDetail {
                            title: "Log",
                            open: false,
                            TributeLog {
                                identifier: tribute.clone().identifier,
                                game_identifier: game_identifier.clone()
                            }
                        }
                    }
                }
            }
        }
        QueryStateData::Settled {
            res: Err(QueryError::TributeNotFound(identifier)),
            ..
        } => {
            rsx! { p { "{identifier} not found." } }
        }
        QueryStateData::Loading { .. } => {
            rsx! { p { "Loading..." } }
        }
        _ => {
            rsx! {}
        }
    }
}

#[component]
fn TributeLog(game_identifier: String, identifier: String) -> Element {
    let log_query = use_query(Query::new(
        (game_identifier.clone(), identifier.clone()),
        TributeLogQ,
    ));
    let reader = log_query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled { res: Ok(logs), .. } => {
            rsx! {
                ul {
                    class: "theme1:text-stone-200 theme2:text-green-200 theme3:text-stone-800",
                    for log in logs {
                        li {
                            p {
                                class: "text-sm",
                                "Day {log.game_day}"
                            }
                            "{log.content}"
                        }
                    }
                }
            }
        }
        QueryStateData::Settled { res: Err(_), .. } => {
            rsx! { p { "Failed to load." }  }
        }
        QueryStateData::Loading { .. } => {
            rsx! { p { "Loading..." }  }
        }
        _ => {
            rsx! {}
        }
    }
}

#[component]
fn TributeAttributes(attributes: Attributes) -> Element {
    rsx! {
        dl {
            class: "grid grid-cols-2 gap-4",
            dt { "Health" }
            dd { "{attributes.health}"}
            dt { "Sanity" }
            dd { "{attributes.sanity}"}
            dt { "Movement" }
            dd { "{attributes.movement}"}
            dt { "Strength" }
            dd { "{attributes.strength}"}
            dt { "Defense" }
            dd { "{attributes.defense}"}
            dt { "Bravery" }
            dd { "{attributes.bravery}"}
            dt { "Intelligence" }
            dd { "{attributes.intelligence}"}
            dt { "Persuasion" }
            dd { "{attributes.persuasion}"}
            dt { "Luck" }
            dd { "{attributes.luck}"}
        }
    }
}

#[component]
fn TributeTraits(traits: Vec<Trait>, turns_since_last_betrayal: u8) -> Element {
    let has_treacherous = traits.contains(&Trait::Treacherous);
    rsx! {
        ul {
            class: "flex flex-row gap-2 flex-wrap",
            for t in traits.iter() {
                li {
                    class: format!(
                        "px-2 py-1 rounded-full text-xs font-semibold capitalize {}",
                        trait_chip_classes(t),
                    ),
                    "{t.label()}"
                }
            }
        }
        if has_treacherous {
            p {
                class: "text-xs mt-2 italic opacity-75",
                "Turns since last betrayal: {turns_since_last_betrayal}"
            }
        }
    }
}

#[component]
fn TributeAllies(game_identifier: String, ally_ids: Vec<uuid::Uuid>) -> Element {
    if ally_ids.is_empty() {
        return rsx! {
            p {
                class: "text-sm italic opacity-75",
                "No allies."
            }
        };
    }

    let roster_query = use_query(Query::new(game_identifier.clone(), GameTributesQ));
    let reader = roster_query.read();
    let state = reader.state();

    match &*state {
        QueryStateData::Settled {
            res: Ok(response), ..
        } => {
            let roster = response.tributes.clone();
            rsx! {
                ul {
                    class: "flex flex-col gap-1",
                    for ally_id in ally_ids.iter() {
                        li {
                            class: "flex flex-row gap-2 items-center text-sm",
                            {
                                let ally = roster.iter().find(|t| &t.id == ally_id);
                                match ally {
                                    Some(t) => {
                                        let dead = matches!(
                                            t.status,
                                            TributeStatus::Dead | TributeStatus::RecentlyDead,
                                        );
                                        let link_class = if dead {
                                            "line-through opacity-60 hover:opacity-100 underline decoration-dotted"
                                        } else {
                                            "hover:underline decoration-2"
                                        };
                                        rsx! {
                                            TributeStatusIcon {
                                                status: t.status.clone(),
                                                css_class: "size-4 theme1:fill-amber-500 theme2:fill-green-200".to_string(),
                                            }
                                            Link {
                                                class: "{link_class}",
                                                to: Routes::TributeDetail {
                                                    game_identifier: game_identifier.clone(),
                                                    tribute_identifier: t.identifier.clone(),
                                                },
                                                "{t.name}"
                                            }
                                        }
                                    }
                                    None => rsx! {
                                        span {
                                            class: "italic opacity-50",
                                            "Unknown"
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
        QueryStateData::Settled { res: Err(_), .. } => rsx! {
            p {
                class: "text-sm italic opacity-75",
                "Failed to load allies."
            }
        },
        QueryStateData::Loading { .. } => rsx! {
            p {
                class: "text-sm italic opacity-75",
                "Loading allies..."
            }
        },
        _ => rsx! {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loyal_uses_green() {
        let c = trait_chip_classes(&Trait::Loyal);
        assert!(c.contains("green-700"), "got {c}");
        assert!(c.contains("border"), "got {c}");
    }

    #[test]
    fn aggressive_uses_orange() {
        let c = trait_chip_classes(&Trait::Aggressive);
        assert!(c.contains("orange-700"), "got {c}");
    }

    #[test]
    fn every_trait_has_classes() {
        // Defensive: missing variants would fail to compile, but make sure
        // we never return an empty string by accident.
        for t in [
            Trait::Loyal,
            Trait::Friendly,
            Trait::Treacherous,
            Trait::Paranoid,
            Trait::LoneWolf,
            Trait::Aggressive,
            Trait::Reckless,
            Trait::Defensive,
            Trait::Cautious,
            Trait::Cunning,
            Trait::Dim,
            Trait::Resilient,
            Trait::Fragile,
            Trait::Tough,
            Trait::Asthmatic,
            Trait::Nearsighted,
        ] {
            let c = trait_chip_classes(&t);
            assert!(!c.is_empty(), "empty for {:?}", t);
            assert!(c.contains("border"), "no border for {:?}", t);
        }
    }
}

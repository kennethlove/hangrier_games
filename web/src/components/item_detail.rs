use crate::cache::QueryError;
use crate::components::icons::uturn::UTurnIcon;
use crate::components::item_icon::ItemIcon;
use crate::http::WithCredentials;
use crate::routes::Routes;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use game::items::Item;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ItemDetailQ;

impl QueryCapability for ItemDetailQ {
    type Ok = Box<Item>;
    type Err = QueryError;
    type Keys = (String, String);

    async fn run(&self, keys: &(String, String)) -> Result<Box<Item>, QueryError> {
        let (game_identifier, item_identifier) = keys;
        let client = reqwest::Client::new();
        let request = client
            .request(
                reqwest::Method::GET,
                crate::api_url::api_url(&format!(
                    "/api/games/{}/items/{}",
                    game_identifier, item_identifier
                )),
            )
            .with_credentials();
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Item>().await {
                        Ok(item) => Ok(Box::new(item)),
                        Err(_) => Err(QueryError::BadJson),
                    }
                } else {
                    Err(QueryError::Unknown)
                }
            }
            Err(_) => Err(QueryError::Unknown),
        }
    }
}

#[component]
pub fn ItemDetail(game_identifier: String, item_identifier: String) -> Element {
    let q = use_query(Query::new(
        (game_identifier.clone(), item_identifier.clone()),
        ItemDetailQ,
    ));
    let reader = q.read();
    let body = match &*reader.state() {
        QueryStateData::Settled { res: Ok(item), .. } => {
            let item = item.as_ref();
            rsx! {
                div {
                    class: "p-4 space-y-3",
                    div {
                        Link {
                            to: Routes::GamePage { identifier: game_identifier.clone() },
                            class: "inline-flex items-center gap-1 text-sm underline",
                            UTurnIcon { class: "size-4 fill-current" }
                            "Back to game"
                        }
                    }
                    div {
                        class: "flex flex-row gap-3 items-center",
                        ItemIcon {
                            item: item.clone(),
                            css_class: "size-10  ",
                        }
                        h1 {
                            class: "text-2xl font-semibold",
                            "{item.name}"
                        }
                    }
                    dl {
                        class: "grid grid-cols-2 gap-x-4 gap-y-1 text-sm",
                        dt { class: "font-semibold", "Type" }
                        dd { "{item.item_type:?}" }
                        dt { class: "font-semibold", "Rarity" }
                        dd { "{item.rarity}" }
                        dt { class: "font-semibold", "Attribute" }
                        dd { "{item.attribute:?}" }
                        dt { class: "font-semibold", "Effect" }
                        dd { "{item.effect}" }
                        dt { class: "font-semibold", "Durability" }
                        dd { "{item.current_durability} / {item.max_durability}" }
                    }
                }
            }
        }
        QueryStateData::Settled { res: Err(_), .. } => {
            rsx! { p { class: "p-4", "Item not found." } }
        }
        QueryStateData::Loading { .. } | QueryStateData::Pending => {
            rsx! { p { class: "p-4", "Loading..." } }
        }
    };
    rsx! { {body} }
}

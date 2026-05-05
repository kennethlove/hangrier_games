use dioxus::prelude::*;
use dioxus_query::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ServerVersionQ;

impl QueryCapability for ServerVersionQ {
    type Ok = String;
    type Err = ();
    type Keys = ();

    async fn run(&self, _keys: &()) -> Result<String, ()> {
        let client = reqwest::Client::new();
        let request = client.request(
            reqwest::Method::GET,
            crate::api_url::api_url("/api/version"),
        );
        match request.send().await {
            Ok(response) => match response.json::<String>().await {
                Ok(version) => Ok(version),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

#[component]
pub fn ServerVersion() -> Element {
    let version_query = use_query(Query::new((), ServerVersionQ));
    let reader = version_query.read();
    let state = reader.state();
    if let Some(version) = state.ok() {
        rsx! { "Server: v{version}" }
    } else {
        rsx! { "Server unavailable" }
    }
}

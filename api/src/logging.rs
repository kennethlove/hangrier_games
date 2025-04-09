use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Debug;
use tracing::{Event, Id};
use tracing::field::Field;
use tracing::span::Attributes;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context};

pub struct HangryGamesLogLayer;

impl<S> Layer<S> for HangryGamesLogLayer
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        attrs.record(&mut visitor);

        let storage = HangryGamesLogStorage(fields);

        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();
        extensions.insert::<HangryGamesLogStorage>(storage);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut spans = vec![];
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let extensions = span.extensions();
                let storage = extensions.get::<HangryGamesLogStorage>().unwrap();
                let field_data: &BTreeMap<String, serde_json::Value> = &storage.0;
                spans.push(serde_json::json!({
                    "target": span.metadata().target(),
                    "name": span.name(),
                    "level": format!("{:?}", span.metadata().level()),
                    "fields": field_data
                }));
            }
        }

        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        event.record(&mut visitor);

        let output = serde_json::json!({
            "target": event.metadata().target(),
            "name": event.metadata().name(),
            "level": format!("{:?}", event.metadata().level()),
            "fields": fields,
            "spans": spans,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}

struct JsonVisitor<'a>(&'a mut BTreeMap<String, serde_json::Value>);

impl tracing::field::Visit for JsonVisitor<'_> {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        self.0.insert(field.name().to_owned(), serde_json::json!(value));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn Error + 'static)) {
        self.0.insert(field.name().to_owned(), serde_json::json!(format!("{:?}", value)));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.0.insert(field.name().to_owned(), serde_json::json!(format!("{:?}", value)));
    }
}

#[derive(Debug)]
struct HangryGamesLogStorage(BTreeMap<String, serde_json::Value>);

use super::*;

impl Game {
    /// Construct a fallback `MessagePayload` when a caller doesn't supply
    /// a typed payload. Picks an existing variant suited to the message
    /// source so the schema-required `payload` field is always present.
    /// This is a transitional helper used by the legacy log helpers
    /// pending full migration of every emission site to typed payloads.
    pub(crate) fn fallback_payload(
        source: &crate::messages::MessageSource,
    ) -> crate::messages::MessagePayload {
        use crate::messages::{AreaEventKind, AreaRef, MessagePayload, MessageSource, TributeRef};
        match source {
            MessageSource::Tribute(id) => MessagePayload::SanityBreak {
                tribute: TributeRef {
                    identifier: id.clone(),
                    name: String::new(),
                },
            },
            MessageSource::Area(name) => MessagePayload::AreaEvent {
                area: AreaRef {
                    identifier: name.clone(),
                    name: name.clone(),
                },
                kind: AreaEventKind::Other,
                description: String::new(),
            },
            MessageSource::Game(_) => MessagePayload::GameEnded { winner: None },
        }
    }

    /// Build and push a `GameMessage` with the supplied typed payload.
    /// Stamps `(game_day, phase, tick, emit_index)` from the game's
    /// transient cycle state. The `tick` argument is supplied by the
    /// caller because some sites (cycle announcements, area events)
    /// emit at the phase boundary (`tick = 0`) while per-tribute
    /// action emissions advance the tick counter.
    pub(crate) fn push_message(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        content: String,
        payload: crate::messages::MessagePayload,
        tick: u32,
    ) {
        let game_day = self.day.unwrap_or(0);
        // Always prefix subject with the game identifier so that the
        // API's per-game log queries (`WHERE string::starts_with(subject,
        // $game_id)`) match every emitted message regardless of source
        // type (Game / Area / Tribute). Without this prefix, area and
        // tribute messages would be invisible to the day-page and
        // timeline-summary endpoints.
        let scoped_subject = if subject.starts_with(&format!("{}:", self.identifier)) {
            subject
        } else {
            format!("{}:{}", self.identifier, subject)
        };
        let msg = crate::messages::GameMessage::new(
            source,
            game_day,
            self.current_phase,
            tick,
            self.emit_index,
            scoped_subject,
            content,
            payload,
        );
        self.messages.push(msg);
        self.emit_index = self.emit_index.saturating_add(1);
    }

    /// Push a message into the cycle's transient event buffer.
    /// The API layer drains and persists this buffer after each cycle.
    ///
    /// This legacy helper synthesises a fallback payload (see
    /// `fallback_payload`) suited to the source. New emission sites
    /// should construct a typed `MessagePayload` and call
    /// [`Self::push_message`] directly.
    pub fn log(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        content: String,
    ) {
        let payload = Self::fallback_payload(&source);
        let tick = self.tick_counter.boundary();
        self.push_message(source, subject, content, payload, tick);
    }

    /// Log a structured game output by rendering its `Display` impl into a `GameMessage`.
    pub fn log_output<D: std::fmt::Display>(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        output: D,
    ) {
        self.log(source, subject, output.to_string());
    }

    /// Legacy helper: log a string output and tag with a typed `MessageKind`.
    /// `kind` is now derived from `MessagePayload::kind()` so the explicit
    /// `kind` argument is ignored — the variant of the synthesised
    /// fallback payload determines the kind. New sites should construct
    /// a typed `MessagePayload` and call [`Self::push_message`] directly.
    pub fn log_output_kind<D: std::fmt::Display>(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        output: D,
        _kind: crate::messages::MessageKind,
    ) {
        self.log(source, subject, output.to_string());
    }

    /// Log a structured [`crate::events::GameEvent`] by rendering its
    /// `Display` impl into the `GameMessage.content`. The typed
    /// `MessagePayload` defaults to a source-appropriate fallback;
    /// callers needing a specific payload variant should instead build
    /// it themselves and call [`Self::push_message`].
    pub fn log_event(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        event: crate::events::GameEvent,
    ) {
        self.log(source, subject, event.to_string());
    }

    /// Log a structured `GameEvent` and tag with `MessageKind`.
    /// As with [`Self::log_output_kind`], the `kind` argument is now
    /// derived from the payload and is accepted for backwards
    /// compatibility only.
    pub fn log_event_kind(
        &mut self,
        source: crate::messages::MessageSource,
        subject: String,
        event: crate::events::GameEvent,
        _kind: crate::messages::MessageKind,
    ) {
        self.log(source, subject, event.to_string());
    }
}

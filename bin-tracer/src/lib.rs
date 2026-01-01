pub mod utils;

use std::{
    collections::HashMap,
    io::Write,
    sync::{Mutex, OnceLock, RwLock, atomic::AtomicU64},
};

use serde::{Deserialize, Serialize};
use tracing_core::{
    Callsite, Dispatch, Event, Interest, Kind, Metadata, Subscriber,
    callsite::{self, DefaultCallsite},
    field::{self, FieldSet},
    identify_callsite, span,
};
use tracing_serde_structured::{AsSerde, SerializeFieldSet, SerializeId, SerializeMetadata};

pub const LOG_PATH: &str = "structured_log";

pub struct SerdeSubscriber {
    callsites: RwLock<Vec<&'static Metadata<'static>>>,
    span_id_counter: AtomicU64, // must be non-zero
    entries: Mutex<Vec<TracingEntries>>,
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TraceStore<'a> {
    callsites: Vec<SerializeMetadata<'a>>,
    entries: Vec<TracingEntries>,
}

struct LateInitCallsite(OnceLock<DefaultCallsite>);

impl Callsite for LateInitCallsite {
    fn set_interest(&self, interest: tracing_core::Interest) {
        self.0
            .get()
            .expect("Callsite impl must not be used before initialization")
            .set_interest(interest)
    }

    fn metadata(&self) -> &Metadata<'_> {
        self.0
            .get()
            .expect("Callsite impl must not be used before initialization")
            .metadata()
    }
}

impl TraceStore<'static> {
    pub fn replay_to_subscriber<S: Subscriber>(&mut self, subscriber: &S) {
        // TODO: don't heap allocate?
        let mut interests = Vec::with_capacity(self.callsites.len());
        let mut callsites = Vec::with_capacity(self.callsites.len());
        for metadata in &self.callsites {
            let callsite = Box::leak(Box::new(LateInitCallsite(OnceLock::new())));

            let SerializeFieldSet::De(cow_strings) = metadata.fields else {
                unreachable!("We just deserialzied the metadata");
            };
            let field_names: Vec<&'static str> = cow_strings
                .into_iter()
                .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
                .collect();
            let field_names = Box::new(field_names).leak();

            let fields = FieldSet::new(&field_names[..], identify_callsite!(callsite));

            let metadata = Metadata::new(
                metadata.name.as_str(),
                metadata.target.as_str(),
                match metadata.level {
                    tracing_serde_structured::SerializeLevel::Trace => tracing_core::Level::TRACE,
                    tracing_serde_structured::SerializeLevel::Debug => tracing_core::Level::DEBUG,
                    tracing_serde_structured::SerializeLevel::Info => tracing_core::Level::INFO,
                    tracing_serde_structured::SerializeLevel::Warn => tracing_core::Level::WARN,
                    tracing_serde_structured::SerializeLevel::Error => tracing_core::Level::ERROR,
                },
                metadata.file.map(|f| f.as_str()),
                metadata.line,
                metadata.module_path.map(|p| p.as_str()),
                fields,
                if metadata.is_event {
                    Kind::EVENT
                } else {
                    Kind::SPAN
                },
            );
            let metadata = Box::leak(Box::new(metadata));
            callsite.0.set(DefaultCallsite::new(metadata)).unwrap();
            callsites.push(callsite);

            let interest = subscriber.register_callsite(metadata);
            // callsite.set_interest(interest); // Should we use the default callsites interest cache here?
            interests.push(interest);
        }
        let mut id_map = std::collections::HashMap::new();
        for action in &self.entries {
            match action {
                TracingEntries::NewSpan {
                    id,
                    metadata_id,
                    values,
                    parent,
                } => {
                    let interest = &interests[*metadata_id];
                    let metadata = callsites[*metadata_id].0.get().unwrap().metadata();
                    // let metadata: &'static Metadata<'static> = todo!(); // TODO: lookup by metadata_id
                    if interest.is_never()
                        || interest.is_sometimes() && !subscriber.enabled(metadata)
                    {
                        continue;
                    }

                    let fields = metadata.fields();
                    let values = fields
                        .iter()
                        .zip(values)
                        .map(|(field, value)| {
                            (
                                &field,
                                value.as_ref().map(|v| v as &dyn tracing_core::field::Value),
                            )
                        })
                        .collect::<Vec<_>>();
                    // let message_field = fields.field("message").unwrap();
                    // #[allow(trivial_casts)] // The compiler is lying, it can't infer this cast
                    // let values = [(
                    //     &message_field,
                    //     Some(&message as &dyn tracing_core::field::Value),
                    // )];

                    // This function is hidden from docs, but we have to use it
                    // because there is no other way of obtaining a `ValueSet`.
                    // It's not entirely clear why it is private. See this issue:
                    // https://github.com/tokio-rs/tracing/issues/2363
                    // TODO: values must be an array of constant size to implement ValidLen
                    // impl<'a, const N: usize> ValidLen<'a> for [(&'a Field, Option<&'a (dyn Value + 'a)>); N] {}
                    // How am I supposed to get that?
                    let values = fields.value_set(&values[..]);
                    // TODO: create a map entry from the returned Id to SerializeId, use it later
                    let span = span::Attributes::new(metadata, &values);
                    let new_id = subscriber.new_span(&span);
                    id_map.insert(id.id, new_id);
                }
                TracingEntries::Event {
                    metadata_id,
                    values,
                    parent,
                } => {
                    let interest = &interests[*metadata_id];
                    let metadata = callsites[*metadata_id].0.get().unwrap().metadata();
                    // let metadata: &'static Metadata<'static> = todo!(); // TODO: lookup by metadata_id
                    if interest.is_never()
                        || interest.is_sometimes() && !subscriber.enabled(metadata)
                    {
                        continue;
                    }

                    let fields = metadata.fields();
                    let values = fields
                        .iter()
                        .zip(values)
                        .map(|(field, value)| {
                            (
                                &field,
                                value.as_ref().map(|v| v as &dyn tracing_core::field::Value),
                            )
                        })
                        .collect::<Vec<_>>();
                    // let message_field = fields.field("message").unwrap();
                    // #[allow(trivial_casts)] // The compiler is lying, it can't infer this cast
                    // let values = [(
                    //     &message_field,
                    //     Some(&message as &dyn tracing_core::field::Value),
                    // )];

                    // This function is hidden from docs, but we have to use it
                    // because there is no other way of obtaining a `ValueSet`.
                    // It's not entirely clear why it is private. See this issue:
                    // https://github.com/tokio-rs/tracing/issues/2363
                    // TODO: values must be an array of constant size to implement ValidLen
                    // impl<'a, const N: usize> ValidLen<'a> for [(&'a Field, Option<&'a (dyn Value + 'a)>); N] {}
                    // How am I supposed to get that?
                    let values = fields.value_set(&values[..]);
                    let event = Event::new(metadata, &values);
                    subscriber.event(&event);
                }
                TracingEntries::Enter { span } => {
                    let id = id_map[&span.id];
                    subscriber.enter(&id);
                }
                TracingEntries::Exit { span } => {
                    let id = id_map[&span.id];
                    subscriber.exit(&id);
                }
                TracingEntries::Record { span, values } => todo!(),
                TracingEntries::RecordFollowsFrom { span, follows } => {
                    let id = id_map[&span.id];
                    let follows_id = id_map[&follows.id];
                    subscriber.record_follows_from(&id, &follows_id);
                }
                TracingEntries::CloneSpan { id } => {
                    let original_id = id_map[&id.id];
                    let _same_id = subscriber.clone_span(&original_id);
                    // id_map.insert(id.id, _same_id); // This should be the same ID
                }
                TracingEntries::TryClose { id } => {
                    let span_id = id_map.remove(&id.id).unwrap();
                    subscriber.try_close(span_id);
                }
            }
        }
    }
}

fn make_field_set(metadata: SerializeMetadata<'_>) -> FieldSet {
    let callsite = Box::leak(Box::new(LateInitCallsite(OnceLock::new())));

    let SerializeFieldSet::De(cow_strings) = metadata.fields else {
        unreachable!("We just deserialzied the metadata");
    };
    let field_names: Vec<&'static str> = cow_strings
        .into_iter()
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
        .collect();
    let field_names = Box::new(field_names).leak();

    let fields = FieldSet::new(&field_names[..], identify_callsite!(callsite));
    callsite.0.set(DefaultCallsite::new(metadata));
    fields
}

impl Drop for SerdeSubscriber {
    fn drop(&mut self) {
        let callsites = self
            .callsites
            .try_read()
            .unwrap()
            .iter()
            .map(|m| m.as_serde())
            .collect();
        let actions = self.entries.lock().unwrap().drain(..).collect();
        let log = TraceStore {
            callsites,
            entries: actions,
        };
        let file = std::fs::File::create(LOG_PATH).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        serde_json::to_writer(&mut writer, &log).unwrap();
        writer.flush().unwrap();
    }
}

impl Default for SerdeSubscriber {
    fn default() -> Self {
        Self {
            callsites: RwLock::new(Vec::new()),
            span_id_counter: AtomicU64::new(1),
            entries: Mutex::new(Vec::new()),
        }
    }
}

// TODO: represent all states
// /// The new span will be a root span.
// Root,
// /// The new span will be rooted in the current span.
// Current,
// /// The new span has an explicitly-specified parent.
// Explicit(Id),
type Parent = Option<SerializeId>;

type Values = Vec<Option<String>>;

#[derive(Serialize, Deserialize, Debug)]
pub enum TracingEntries {
    NewSpan {
        id: SerializeId,
        metadata_id: usize,
        values: Values,
        parent: Parent,
    },
    Event {
        metadata_id: usize,
        values: Values,
        parent: Parent,
    },
    Enter {
        span: SerializeId,
    },
    Exit {
        span: SerializeId,
    },
    Record {
        span: SerializeId,
        values: Values,
    },
    RecordFollowsFrom {
        span: SerializeId,
        follows: SerializeId,
    },
    CloneSpan {
        id: SerializeId,
    },
    TryClose {
        id: SerializeId,
    },
}

struct FieldVisitor {
    fields: Values,
}

impl tracing_core::field::Visit for FieldVisitor {
    fn record_debug(&mut self, _field: &tracing_core::Field, value: &dyn std::fmt::Debug) {
        self.fields.push(Some(format!("{value:?}")));
    }
    // implement visit methods
}

impl Subscriber for SerdeSubscriber {
    // Not used by us, we want all events and spans
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> tracing_core::Interest {
        // Here is where the magic happens, we need to store the callsite metadata somewhere
        self.callsites.write().unwrap().push(metadata);
        tracing_core::Interest::always()
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        // Just generate a new span ID, don't equate spans by their metadata or fields, for maximum flexibility
        let id = span::Id::from_u64(
            self.span_id_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );

        self.entries.lock().unwrap().push(TracingEntries::NewSpan {
            id: id.as_serde(),
            metadata_id: self
                .callsites
                .read()
                .unwrap()
                .iter()
                .position(|m| *m == span.metadata())
                .unwrap(),
            values: {
                let len = span.fields().len();
                let mut visitor = FieldVisitor {
                    fields: Vec::with_capacity(len),
                };
                span.record(&mut visitor);
                visitor.fields
            },
            parent: span.parent().map(|p| p.as_serde()),
        });
        id
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        self.entries.lock().unwrap().push(TracingEntries::Record {
            span: span.as_serde(),
            values: {
                let len = values.len();
                let mut visitor = FieldVisitor {
                    fields: Vec::with_capacity(len),
                };
                values.record(&mut visitor);
                visitor.fields
            },
        });
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        self.entries
            .lock()
            .unwrap()
            .push(TracingEntries::RecordFollowsFrom {
                span: span.as_serde(),
                follows: follows.as_serde(),
            });
    }

    fn event(&self, event: &tracing_core::Event<'_>) {
        // TODO: optimize lookup
        let metadata_id = self
            .callsites
            .read()
            .unwrap()
            .iter()
            .position(|m| *m == event.metadata())
            .unwrap();

        let len = event.fields().count();
        let mut visitor = FieldVisitor {
            fields: Vec::with_capacity(len),
        };
        event.record(&mut visitor);

        self.entries.lock().unwrap().push(TracingEntries::Event {
            metadata_id,
            values: visitor.fields,
            parent: event.parent().map(|p| p.as_serde()),
        });
    }

    fn enter(&self, span: &span::Id) {
        self.entries.lock().unwrap().push(TracingEntries::Enter {
            span: span.as_serde(),
        });
    }

    fn exit(&self, span: &span::Id) {
        self.entries.lock().unwrap().push(TracingEntries::Exit {
            span: span.as_serde(),
        });
    }

    fn clone_span(&self, id: &span::Id) -> span::Id {
        self.entries
            .lock()
            .unwrap()
            .push(TracingEntries::CloneSpan { id: id.as_serde() });
        id.clone()
    }

    fn try_close(&self, id: span::Id) -> bool {
        self.entries
            .lock()
            .unwrap()
            .push(TracingEntries::TryClose { id: id.as_serde() });
        false
    }

    // fn drop_span(&self, id: span::Id) {
    //     self.actions
    //         .lock()
    //         .unwrap()
    //         .push(TracingAction::DropSpan { id: id.as_serde() });
    // }

    // Don't think we need these
    // fn on_register_dispatch(&self, subscriber: &tracing_core::Dispatch) {
    //     let _ = subscriber;
    // }

    // fn max_level_hint(&self) -> Option<tracing_core::LevelFilter> {
    //     None
    // }

    // fn event_enabled(&self, event: &tracing_core::Event<'_>) -> bool {
    //     let _ = event;
    //     true
    // }

    // fn current_span(&self) -> tracing_core::span::Current {
    //     tracing_core::span::Current::unknown()
    // }

    // unsafe fn downcast_raw(&self, id: std::any::TypeId) -> Option<*const ()> {
    //     if id == std::any::TypeId::of::<Self>() {
    //         Some(self as *const Self as *const ())
    //     } else {
    //         None
    //     }
    // }
}

pub mod utils;

use std::{
    io::Write,
    sync::{Mutex, RwLock, atomic::AtomicU64},
};

use serde::{Deserialize, Serialize};
use tracing_core::{
    Dispatch, Kind, Metadata, Subscriber,
    callsite::{self, DefaultCallsite},
    span,
};
use tracing_serde_structured::{AsSerde, SerializeFieldSet, SerializeId, SerializeMetadata};

pub const LOG_PATH: &str = "structured_log";

pub struct BinarySubscriber {
    callsites: RwLock<Vec<&'static Metadata<'static>>>,
    span_id_counter: AtomicU64, // must be non-zero
    actions: Mutex<Vec<TracingAction>>,
}

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct BinaryLog<'a> {
    callsites: Vec<SerializeMetadata<'a>>,
    actions: Vec<TracingAction>,
}

impl BinaryLog<'static> {
    pub fn replay_to_subscriber<S: Subscriber>(&mut self, subscriber: &S) {
        for callsite in self.callsites.drain(..) {
            let metadata = Metadata::new(
                callsite.name.as_str(),
                callsite.target.as_str(),
                match callsite.level {
                    tracing_serde_structured::SerializeLevel::Trace => tracing_core::Level::TRACE,
                    tracing_serde_structured::SerializeLevel::Debug => tracing_core::Level::DEBUG,
                    tracing_serde_structured::SerializeLevel::Info => tracing_core::Level::INFO,
                    tracing_serde_structured::SerializeLevel::Warn => tracing_core::Level::WARN,
                    tracing_serde_structured::SerializeLevel::Error => tracing_core::Level::ERROR,
                },
                callsite.file.map(|f| f.as_str()),
                callsite.line,
                callsite.module_path.map(|p| p.as_str()),
                callsite.fields,
                if callsite.is_event {
                    Kind::EVENT
                } else {
                    Kind::SPAN
                },
            );
            let metadata = Box::new(metadata);
            // TODO: Figure out how to map SerializeMetadata<'static> to a &'static Metadata<'static>
            let _ = subscriber.register_callsite(Box::leak(metadata));
        }
        for action in &self.actions {
            match action {
                TracingAction::NewSpan {
                    id,
                    metadata_id,
                    values,
                    parent,
                } => subscriber.new_span(span),
                TracingAction::Event {
                    metadata_id,
                    values,
                    parent,
                } => todo!(),
                TracingAction::Enter { span } => todo!(),
                TracingAction::Exit { span } => todo!(),
                TracingAction::Record { span, values } => todo!(),
                TracingAction::RecordFollowsFrom { span, follows } => todo!(),
                TracingAction::CloneSpan { id } => todo!(),
                TracingAction::DropSpan { id } => todo!(),
            }
        }
    }
}

impl Drop for BinarySubscriber {
    fn drop(&mut self) {
        let callsites = self
            .callsites
            .try_read()
            .unwrap()
            .iter()
            .map(|m| m.as_serde())
            .collect();
        let actions = self.actions.lock().unwrap().drain(..).collect();
        let log = BinaryLog { callsites, actions };
        let file = std::fs::File::create(LOG_PATH).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        serde_json::to_writer(&mut writer, &log).unwrap();
        writer.flush().unwrap();
    }
}

impl Default for BinarySubscriber {
    fn default() -> Self {
        Self {
            callsites: RwLock::new(Vec::new()),
            span_id_counter: AtomicU64::new(1),
            actions: Mutex::new(Vec::new()),
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
pub enum TracingAction {
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
    DropSpan {
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

impl Subscriber for BinarySubscriber {
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

        self.actions.lock().unwrap().push(TracingAction::NewSpan {
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
        self.actions.lock().unwrap().push(TracingAction::Record {
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
        self.actions
            .lock()
            .unwrap()
            .push(TracingAction::RecordFollowsFrom {
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

        self.actions.lock().unwrap().push(TracingAction::Event {
            metadata_id,
            values: visitor.fields,
            parent: event.parent().map(|p| p.as_serde()),
        });
    }

    fn enter(&self, span: &span::Id) {
        self.actions.lock().unwrap().push(TracingAction::Enter {
            span: span.as_serde(),
        });
    }

    fn exit(&self, span: &span::Id) {
        self.actions.lock().unwrap().push(TracingAction::Exit {
            span: span.as_serde(),
        });
    }

    fn clone_span(&self, id: &span::Id) -> span::Id {
        self.actions
            .lock()
            .unwrap()
            .push(TracingAction::CloneSpan { id: id.as_serde() });
        id.clone()
    }

    fn drop_span(&self, id: span::Id) {
        self.actions
            .lock()
            .unwrap()
            .push(TracingAction::DropSpan { id: id.as_serde() });
    }

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

    // fn try_close(&self, id: span::Id) -> bool {
    //     #[allow(deprecated)]
    //     self.drop_span(id);
    //     false
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

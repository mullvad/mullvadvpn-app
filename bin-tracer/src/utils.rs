use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock as OnceCell},
};

// use once_cell::sync::OnceCell;
use tracing_core::{
    Callsite, Level, LevelFilter, Metadata, callsite::DefaultCallsite, field::FieldSet,
};
use tracing_core::{identify_callsite, metadata::Kind as MetadataKind};

/// Log an event.
///
/// The target should be something like a module path, and can be referenced in
/// the filter string given to `init_platform`. `level` and `target` for a
/// callsite are fixed at the first `log_event` call for that callsite and can
/// not be changed afterwards, i.e. the level and target passed for second and
/// following `log_event`s with the same callsite will be ignored.
///
/// This function leaks a little bit of memory for each unique (file + line +
/// level + target) it is called with. Please make sure that the number of
/// different combinations of those parameters this can be called with is
/// constant in the final executable.
fn log_event(file: String, line: Option<u32>, level: LogLevel, target: String, message: String) {
    static CALLSITES: Mutex<BTreeMap<MetadataId, &'static DefaultCallsite>> =
        Mutex::new(BTreeMap::new());

    let id = MetadataId {
        file,
        line,
        level,
        target,
        name: None,
    };
    let callsite = get_or_init_metadata(&CALLSITES, id, &["message"], MetadataKind::EVENT);

    if span_or_event_enabled(callsite) {
        let metadata = callsite.metadata();
        let fields = metadata.fields();
        let message_field = fields.field("message").unwrap();
        #[allow(trivial_casts)] // The compiler is lying, it can't infer this cast
        let values = [(
            &message_field,
            Some(&message as &dyn tracing_core::field::Value),
        )];

        // This function is hidden from docs, but we have to use it
        // because there is no other way of obtaining a `ValueSet`.
        // It's not entirely clear why it is private. See this issue:
        // https://github.com/tokio-rs/tracing/issues/2363
        let values = fields.value_set(&values);
        tracing_core::Event::dispatch(metadata, &values);
    }
}

type FieldNames = &'static [&'static str];

fn get_or_init_metadata(
    mutex: &Mutex<BTreeMap<MetadataId, &'static DefaultCallsite>>,
    id: MetadataId,
    field_names: FieldNames,
    meta_kind: MetadataKind,
) -> &'static DefaultCallsite {
    mutex.lock().unwrap().entry(id).or_insert_with_key(|id| {
        let callsite = Box::leak(Box::new(LateInitCallsite(OnceCell::new())));
        let metadata = Box::leak(Box::new(Metadata::new(
            Box::leak(
                id.name
                    .clone()
                    .unwrap_or_else(|| match id.line {
                        Some(line) => format!("event {}:{line}", id.file),
                        None => format!("event {}", id.file),
                    })
                    .into_boxed_str(),
            ),
            Box::leak(id.target.as_str().into()),
            id.level.to_tracing_level(),
            Some(Box::leak(Box::from(id.file.as_str()))),
            id.line,
            None, // module path
            FieldSet::new(field_names, identify_callsite!(callsite)),
            meta_kind,
        )));
        callsite.0.get_or_init(|| DefaultCallsite::new(metadata))
    })
}

const STATIC_MAX_LEVEL: Level = Level::TRACE;

fn span_or_event_enabled(callsite: &'static DefaultCallsite) -> bool {
    let meta = callsite.metadata();
    let level = *meta.level();

    if level > STATIC_MAX_LEVEL || level > LevelFilter::current() {
        false
    } else {
        let interest = callsite.interest();
        interest.is_always()
            || !interest.is_never()
                && tracing_core::dispatcher::get_default(|default| default.enabled(meta))
    }
}

pub struct Span(tracing_core::span::Span);

impl Span {
    /// Create a span originating at the given callsite (file, line and column).
    ///
    /// The target should be something like a module path, and can be referenced
    /// in the filter string given to `setup_tracing`. `level` and `target`
    /// for a callsite are fixed at the first creation of a span for that
    /// callsite and can not be changed afterwards, i.e. the level and
    /// target passed for second and following creation of a span with the same
    /// callsite will be ignored.
    ///
    /// This function leaks a little bit of memory for each unique (file +
    /// line + level + target + name) it is called with. Please make sure that
    /// the number of different combinations of those parameters this can be
    /// called with is constant in the final executable.
    ///
    /// For a span to have an effect, you must `.enter()` it at the start of a
    /// logical unit of work and `.exit()` it at the end of the same (including
    /// on failure). Entering registers the span in thread-local storage, so
    /// future calls to `log_event` on the same thread are able to attach the
    /// events they create to the span, exiting unregisters it. For this to
    /// work, exiting a span must be done on the same thread where it was
    /// entered. It is possible to enter a span on multiple threads, in which
    /// case it should also be exited on all of them individually; that is,
    /// unless you *want* the span to be attached to all further events created
    /// on that thread.
    pub fn new(
        file: String,
        line: Option<u32>,
        level: LogLevel,
        target: String,
        name: String,
    ) -> Arc<Self> {
        static CALLSITES: Mutex<BTreeMap<MetadataId, &'static DefaultCallsite>> =
            Mutex::new(BTreeMap::new());

        let loc = MetadataId {
            file,
            line,
            level,
            target,
            name: Some(name),
        };
        let callsite = get_or_init_metadata(&CALLSITES, loc, &[], MetadataKind::SPAN);
        let metadata = callsite.metadata();

        let span = if span_or_event_enabled(callsite) {
            // This function is hidden from docs, but we have to use it (see above).
            let values = metadata.fields().value_set(&[]);
            Span::new(metadata, &values)
        } else {
            Span::none()
        };

        Arc::new(Self(span))
    }

    pub fn current() -> Arc<Self> {
        Arc::new(Self(Span::current()))
    }

    fn enter(&self) {
        self.0.with_subscriber(|(id, dispatch)| dispatch.enter(id));
    }

    fn exit(&self) {
        self.0.with_subscriber(|(id, dispatch)| dispatch.exit(id));
    }

    fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn to_tracing_level(self) -> Level {
        match self {
            LogLevel::Error => Level::ERROR,
            LogLevel::Warn => Level::WARN,
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct MetadataId {
    file: String,
    line: Option<u32>,
    level: LogLevel,
    target: String,
    name: Option<String>,
}

struct LateInitCallsite(OnceCell<DefaultCallsite>);

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

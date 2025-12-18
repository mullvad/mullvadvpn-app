use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use libc::exit;
use tracing::{Level, subscriber};
use tracing::{Subscriber, Value};
use tracing_serde_structured::{SerializeEvent, SerializeRecordFields};
use tracing_subscriber::{EnvFilter, fmt, util::SubscriberInitExt};

use mullvad_daemon::logging::binary_logger::LOG_PATH;

use tracing::{
    Dispatch, Metadata,
    callsite::DefaultCallsite,
    field::{Field, FieldSet},
};
use tracing_core::{identify_callsite, metadata::Kind};

struct LateInitCallsite(OnceLock<DefaultCallsite>);
impl tracing::Callsite for LateInitCallsite {
    fn set_interest(&self, i: tracing_core::Interest) {
        self.0.get().unwrap().set_interest(i)
    }
    fn metadata(&self) -> &tracing::Metadata<'_> {
        self.0.get().unwrap().metadata()
    }
}

fn main() -> Result<()> {
    // Pretty-print to stderr; control verbosity with RUST_LOG
    let sub = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .with_writer(std::io::stderr)
        .finish();

    replay(LOG_PATH, sub)
}

fn replay(path: &str, subscriber: impl tracing::Subscriber) -> Result<()> {
    let f = File::open(path).with_context(|| format!("open {path}"))?;
    let rdr = BufReader::new(f);

    for (i, line) in rdr.lines().enumerate() {
        let line = line?;
        if let Ok(v) =
            serde_json::from_str(&line).with_context(|| format!("parse json line {}", i + 1))
        {
            emit_event(&v, subscriber);
            return Ok(());
        }
    }
    Ok(())
}

fn emit_event(_event: &SerializeEvent<'_>, subscriber: impl tracing::Subscriber) {
    // dbg!(event);
    // let SerializeEvent {
    //     fields,
    //     metadata,
    //     parent,
    // } = event;
    // let SerializeRecordFields::De(record_map) = fields else {
    //     panic!()
    // };

    // 1) Make/leak 'static strings (yeah, for a replay CLI this is fine).
    let name: &'static str = Box::leak("event foo.rs:12".to_string().into_boxed_str());
    let target: &'static str = Box::leak("my_mod".to_string().into_boxed_str());
    let file: Option<&'static str> = Some(Box::leak("foo.rs".to_string().into_boxed_str()));
    let module_path: Option<&'static str> = None;

    // 2) Build a callsite + field set
    static CALLSITE: LateInitCallsite = LateInitCallsite(OnceLock::new());
    static FIELD_NAMES: &[&str] = &["message", "user_id", "latency_ms"];
    let fields = FieldSet::new(FIELD_NAMES, identify_callsite!(&CALLSITE));
    let meta = Box::leak(Box::new(Metadata::new(
        name,
        target,
        Level::INFO,
        file,
        Some(12),
        module_path,
        fields,
        Kind::EVENT,
    )));
    CALLSITE.0.set(DefaultCallsite::new(meta)).ok();
    CALLSITE.0.get().unwrap().register(); // or .interest() which registers lazily

    // 3) Build a ValueSet for this event instance
    let keys = meta.fields();
    let msg = "user logged in";
    let user_id: i64 = 42;
    let latency_ms: u64 = 12;
    // pairs are (&Field, Option<&dyn Value>)
    let values_arr = [
        (&keys.field("message").unwrap(), Some(&msg as &dyn Value)),
        (&keys.field("user_id").unwrap(), Some(&user_id)),
        (&keys.field("latency_ms").unwrap(), Some(&latency_ms)),
    ];
    let values = keys.value_set(&values_arr); // doc(hidden) but public

    // 4) Construct and deliver the event
    let event = tracing::Event::new(meta, &values);

    subscriber.event(&event);
}

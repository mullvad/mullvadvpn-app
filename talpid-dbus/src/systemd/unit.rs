#[derive(Debug)]
pub struct Unit {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub follow_unit: String,
    pub object_path: dbus::strings::Path<'static>,
    pub job_id: u32,
    pub job_type: String,
    pub job_object_path: dbus::strings::Path<'static>,
}

impl Unit {
    pub fn is_running(&self) -> bool {
        self.sub_state == "running"
    }

    pub fn is_active(&self) -> bool {
        self.active_state == "active"
    }
}

impl dbus::arg::Arg for Unit {
    const ARG_TYPE: dbus::arg::ArgType = dbus::arg::ArgType::Struct;
    fn signature() -> dbus::Signature<'static> {
        dbus::Signature::from_slice("a(ssssssouso)")
            .expect("Failed to construct type signature for Unit")
    }
}

impl<'a> dbus::arg::Get<'a> for Unit {
    fn get(i: &mut dbus::arg::Iter<'a>) -> Option<Self> {
        let mut i = i.recurse(dbus::arg::ArgType::Struct)?;

        Some(Self {
            name: i.read().ok()?,
            description: i.read().ok()?,
            load_state: i.read().ok()?,
            active_state: i.read().ok()?,
            sub_state: i.read().ok()?,
            follow_unit: i.read().ok()?,
            object_path: i.read().ok()?,
            job_id: i.read().ok()?,
            job_type: i.read().ok()?,
            job_object_path: i.read().ok()?,
        })
    }
}

#[test]
fn test_dbus_signature() {
    let _ = <Unit as dbus::arg::Arg>::signature();
}

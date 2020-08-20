use mullvad_management_interface::types::KeygenEvent;

pub fn print_keygen_event(key_event: &KeygenEvent) {
    use mullvad_management_interface::types::keygen_event::KeygenEvent as EventType;

    match EventType::from_i32(key_event.event).unwrap() {
        EventType::NewKey => {
            println!(
                "New WireGuard key: {}",
                base64::encode(&key_event.new_key.as_ref().unwrap().key)
            );
        }
        EventType::TooManyKeys => {
            println!("Account has too many keys already");
        }
        EventType::GenerationFailure => {
            println!("Failed to generate new WireGuard key");
        }
    }
}

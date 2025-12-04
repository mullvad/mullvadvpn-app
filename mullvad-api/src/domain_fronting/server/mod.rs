
struct Sessions {
    sessions: papaya::HashMap<Uuid, mpsc::Sender<SessionCommand>,
}

struct Session {
    connection: TcpStream,
    received_chunk: Option<Vec<u8>>,
    cmd_rx: mpsc::Receiver<SessionCommand>,
}

impl Session {

}

struct SessionCommand {
    tx_payload: Option<Vec<u8>>,
    rx: mpsc::Sender<Option<Vec<u8>>,
}

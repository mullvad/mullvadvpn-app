use futures::StreamExt;
use pcap::{Device, Packet, PacketCodec, PacketHeader};
use std::{collections::BTreeMap, io, path::PathBuf, sync::mpsc as sync_mpsc};
use tokio::{fs::File, io::BufReader, sync::oneshot};

mod parse;
pub use parse::parse_pcap;
mod cleanup;
pub use cleanup::delete_old_captures;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The capture is in progress")]
    CaptureInProgress,
    #[error("Failed to capture handle for device")]
    OpenHandle(#[source] pcap::Error),
    #[error("Failed to make capture nonblocking")]
    EnableNonblock(#[source] pcap::Error),
    #[error("Failed to begin capture")]
    BeginCapture(#[source] pcap::Error),
    #[error("Failed to create pcap file")]
    CreateDump(#[source] pcap::Error),
    #[error("Failed to create packet stream")]
    CreateStream(#[source] pcap::Error),
    #[error("Packet stream returned an error")]
    StreamFailed(#[source] pcap::Error),
    #[error("Could not find the specified label")]
    CaptureNotFound,
    #[error("Failed to open pcap file")]
    ReadPcap(#[source] io::Error),
}

// Maximum capture size should be 100mb
const MAX_CAPTURE_SIZE: u32 = 1024 * 1024 * 100;

#[derive(Default)]
pub struct Capture {
    captures: BTreeMap<uuid::Uuid, Context>,
}

struct Context {
    capture: tokio::task::JoinHandle<Result<(), Error>>,
    stop_tx: oneshot::Sender<()>,
}

pub struct CloneCodec;

impl PacketCodec for CloneCodec {
    type Item = (PacketHeader, Box<[u8]>);

    fn decode(&mut self, packet: Packet) -> Self::Item {
        (packet.header.to_owned(), packet.data.into())
    }
}

const RAAS_TMP_DIR: &str = "raas";

impl Capture {
    fn capture_file_path(label: &uuid::Uuid) -> PathBuf {
        Self::capture_dir_path().join(label.to_string())
    }

    fn capture_dir_path() -> PathBuf {
        std::env::temp_dir().join(RAAS_TMP_DIR)
    }

    pub async fn start(&mut self, label: uuid::Uuid) -> Result<(), Error> {
        if self.captures.contains_key(&label) {
            return Err(Error::CaptureInProgress);
        }

        // Use the magic any device.
        // This will remove the ethernet frames from the packets.
        let device = Device {
            name: "any".into(),
            desc: None,
            addresses: vec![],
            flags: pcap::DeviceFlags::empty(),
        };

        let capture = pcap::Capture::from_device(device)
            .map_err(Error::OpenHandle)?
            .immediate_mode(true)
            .open()
            .map_err(Error::BeginCapture)?
            .setnonblock()
            .map_err(Error::EnableNonblock)?;

        let dump_path = Self::capture_file_path(&label);
        let mut dump = capture.savefile(dump_path).map_err(Error::CreateDump)?;

        let (stop_tx, mut stop_rx) = oneshot::channel();

        let mut stream = capture.stream(CloneCodec).map_err(Error::CreateStream)?;

        let capture = tokio::spawn(async move {
            #[allow(clippy::type_complexity)]
            let (pcap_tx, pcap_rx): (_, sync_mpsc::Receiver<(PacketHeader, Box<[u8]>)>) =
                sync_mpsc::channel();
            tokio::task::spawn_blocking(move || {
                while let Ok((header, data)) = pcap_rx.recv() {
                    let packet = Packet {
                        header: &header,
                        data: &data,
                    };
                    dump.write(&packet);
                }
                if let Err(error) = dump.flush() {
                    log::error!("Failed to flush pcap dump: {error}");
                }
            });

            let mut capture_size = 0;
            loop {
                tokio::select! {
                    _ = &mut stop_rx => {
                        break;
                    }
                    packet = stream.next() => {
                        let Some(result) = packet else {
                            break;
                        };
                        let (header, data) = result.map_err(Error::StreamFailed)?;
                        let _ = pcap_tx.send((header, data));
                        capture_size += header.caplen;

                        if capture_size >= MAX_CAPTURE_SIZE {
                            break;
                        }
                    }
                }
            }

            Ok(())
        });

        let context = Context { capture, stop_tx };

        self.captures.insert(label, context);

        Ok(())
    }

    pub async fn stop(&mut self, label: uuid::Uuid) -> Result<(), Error> {
        if let Some(context) = self.captures.remove(&label) {
            let _ = context.stop_tx.send(());
            if let Ok(result) = context.capture.await {
                result?;
            }
        }
        Ok(())
    }

    pub async fn get(&self, label: uuid::Uuid) -> Result<BufReader<File>, Error> {
        if self.captures.contains_key(&label) {
            return Err(Error::CaptureInProgress);
        }

        let dump_path = Self::capture_file_path(&label);
        if !dump_path.exists() {
            return Err(Error::CaptureNotFound);
        }

        let file = File::open(dump_path).await.map_err(Error::ReadPcap)?;
        Ok(BufReader::new(file))
    }
}

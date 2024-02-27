use futures::StreamExt;
use pcap::{Device, Packet, PacketCodec, PacketHeader};
use std::collections::BTreeMap;
use std::io;
use std::sync::mpsc as sync_mpsc;
use tokio::sync::oneshot;
use tokio::{
    fs::File,
    io::BufReader,
};
use tokio_util::io::ReaderStream;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The capture is in progress")]
    CaptureInProgress,
    #[error("Failed to look up device")]
    DeviceLookup(#[source] pcap::Error),
    #[error("No device found")]
    NoDeviceFound,
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

impl Capture {
    pub async fn start(&mut self, label: uuid::Uuid) -> Result<(), Error> {
        if self.captures.contains_key(&label) {
            return Err(Error::CaptureInProgress);
        }

        let device = Device::lookup()
            .map_err(Error::DeviceLookup)?
            .ok_or(Error::NoDeviceFound)?;

        let capture = pcap::Capture::from_device(device)
            .map_err(Error::OpenHandle)?
            .immediate_mode(true)
            .open()
            .map_err(Error::BeginCapture)?
            .setnonblock()
            .map_err(Error::EnableNonblock)?;

        let dump_path = std::env::temp_dir().join(label.to_string());
        let mut dump = capture.savefile(&dump_path).map_err(Error::CreateDump)?;

        let (stop_tx, mut stop_rx) = oneshot::channel();

        let mut stream = capture.stream(CloneCodec).map_err(Error::CreateStream)?;

        let capture = tokio::spawn(async move {
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
                    }
                }
            }

            Ok(())
        });

        let context = Context {
            capture,
            stop_tx,
        };

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

    pub async fn get(&self, label: uuid::Uuid) -> Result<ReaderStream<BufReader<File>>, Error> {
        if self.captures.contains_key(&label) {
            return Err(Error::CaptureInProgress);
        }

        let dump_path = std::env::temp_dir().join(label.to_string());
        if !dump_path.exists() {
            return Err(Error::CaptureNotFound);
        }

        let file = File::open(dump_path).await.map_err(Error::ReadPcap)?;
        Ok(ReaderStream::new(BufReader::new(file)))
    }
}

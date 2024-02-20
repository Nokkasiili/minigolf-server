use anyhow::{bail, Context, Result};
use flume::{Receiver, Sender};
use futures_lite::FutureExt;
use protocol::{
    client::ClientToServer,
    common::{Packet, Parse},
    server::ServerToClient,
};
use std::ops::Add;
use std::{fmt::Debug, io, time::Duration};
use std::{io::ErrorKind, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    time::timeout,
};

use crate::{
    codec::MinigolfCodec,
    initial_handler::{self, InitialHandling, NewPlayer},
    playerid::IdGenerator,
};

pub struct Listener {
    listener: TcpListener,
    new_players: Sender<NewPlayer>,
    id_generator: IdGenerator,
}

pub struct Worker {
    reader: Reader,
    writer: Writer,
    packets_to_send_tx: Sender<ServerToClient>,
    received_packets_rx: Receiver<ClientToServer>,
    new_players: Sender<NewPlayer>,
    id_generator: IdGenerator,
}
impl Worker {
    fn new(
        stream: TcpStream,
        addr: SocketAddr,
        new_players: Sender<NewPlayer>,
        id_generator: IdGenerator,
    ) -> Worker {
        let (reader, writer) = stream.into_split();

        let (received_packets_tx, received_packets_rx) = flume::bounded(32);
        let (packets_to_send_tx, packets_to_send_rx) = flume::unbounded();
        let reader = Reader::new(reader, received_packets_tx);
        let writer = Writer::new(writer, packets_to_send_rx);

        Worker {
            reader,
            writer,
            packets_to_send_tx,
            received_packets_rx,
            new_players,
            id_generator,
        }
    }
    pub fn start(self) {
        tokio::task::spawn(async move {
            match self.run().await {
                Ok(_) => {}
                Err(e) => log::debug!("failed initial handling"),
            }
        });
    }
    async fn run(mut self) -> Result<()> {
        let result = initial_handler::handle(&mut self).await;
        match result {
            Ok(result) => self.proceed(result).await,
            Err(e) => log::debug!("Initial handling failed: {:?}", e),
        }
        Ok(())
    }

    async fn proceed(self, result: InitialHandling) {
        match result {
            InitialHandling::Join(new_player) => {
                let name = new_player.name.clone();
                let _ = self.new_players.send_async(new_player).await;
                self.split(name);
            }
        }
    }

    pub fn split(self, username: String) {
        let Self { reader, writer, .. } = self;
        let reader = tokio::task::spawn(async move { reader.run().await });
        let writer = tokio::task::spawn(async move { writer.run().await });

        tokio::task::spawn(async move {
            let result = reader.race(writer).await.expect("task panicked");
            if let Err(e) = result {
                let message = disconnected_message(e);
                log::debug!("{} lost connection: {}", username, message);
            }
            //            player_count.remove_player();
        });
    }

    pub async fn read<T>(&mut self) -> anyhow::Result<T>
    where
        T: Parse + Packet,
    {
        self.reader.read().await
    }

    pub async fn write<D: Debug + Packet + Parse>(&mut self, packet: D) -> anyhow::Result<()> {
        self.writer.write(packet).await
    }
    pub async fn write_str(&mut self, str: &str) -> anyhow::Result<()> {
        self.writer.write_str(&str).await
    }

    pub fn id_generator(&mut self) -> IdGenerator {
        self.id_generator.clone()
    }

    pub fn packets_to_send(&mut self) -> Sender<ServerToClient> {
        self.packets_to_send_tx.clone()
    }

    pub fn received_packets(&mut self) -> Receiver<ClientToServer> {
        self.received_packets_rx.clone()
    }
}

impl Listener {
    pub async fn start(new_players: Sender<NewPlayer>, id_generator: IdGenerator) -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:4242")
            .await
            .context("failed to bind to port - maybe a server is already running?")?;
        let listener = Listener {
            listener,
            new_players,
            id_generator,
        };

        log::info!("Server is listening on :4242",);
        tokio::spawn(async move {
            listener.run().await;
        });
        Ok(())
    }

    async fn run(mut self) {
        loop {
            if let Ok((stream, addr)) = self.listener.accept().await {
                log::info!("Accepted {}", addr);
                self.accept(stream, addr).await;
            }
        }
    }
    async fn accept(&mut self, stream: TcpStream, addr: SocketAddr) {
        let worker = Worker::new(
            stream,
            addr,
            self.new_players.clone(),
            self.id_generator.clone(),
        );
        worker.start();
    }
}

struct Reader {
    stream: OwnedReadHalf,
    buffer: [u8; 512],
    received_packets: Sender<ClientToServer>,
    read: u32,
    codec: MinigolfCodec,
}

impl Reader {
    pub fn new(stream: OwnedReadHalf, received_packets: Sender<ClientToServer>) -> Self {
        Self {
            stream,
            buffer: [0; 512],
            received_packets,
            read: 3, //TODO
            codec: MinigolfCodec::new(),
        }
    }
    pub fn add_num(&mut self) -> u32 {
        self.read = self.read.add(1);
        self.read
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let packet: ClientToServer = self.read().await?;
            if let Some(number) = packet.packet_number() {
                if number.0 != self.add_num() {
                    log::error!("Wrong packet_number {:?} {:?}\n", number.0, self.read);
                }
            }
            log::trace!("recv :{:?}", packet);
            let result = self.received_packets.send_async(packet).await;

            if result.is_err() {
                // server dropped connection
                return Ok(());
            }
        }
    }

    pub async fn read<T>(&mut self) -> anyhow::Result<T>
    where
        T: Parse + Packet,
    {
        loop {
            if let Some(packet) = self.codec.next_packet::<T>()? {
                return Ok(packet);
            }

            let duration = Duration::from_secs(10);

            let bytes_read = timeout(duration, self.stream.read(&mut self.buffer)).await??;

            if bytes_read == 0 {
                //return Err(anyhow::Error::new(ErrorKind::UnexpectedEof, "read 0 bytes"));
                bail!("read 0 bytes");
            }
            let bytes = &self.buffer[..bytes_read];

            self.codec.accept(&bytes);
        }
    }
}

struct Writer {
    stream: OwnedWriteHalf,
    packets_to_send: Receiver<ServerToClient>,
    buffer: Vec<u8>,
}

impl Writer {
    pub fn new(stream: OwnedWriteHalf, packets_to_send: Receiver<ServerToClient>) -> Self {
        Self {
            stream,
            packets_to_send,
            buffer: Vec::new(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Ok(packet) = self.packets_to_send.recv_async().await {
            self.write(packet).await?;
        }
        Ok(())
    }

    pub async fn write(&mut self, packet: impl Parse + Packet + Debug) -> anyhow::Result<()> {
        log::trace!("send: {:?}", packet);
        self.buffer = packet.as_string().into();
        self.stream.write_all(&self.buffer).await?;
        self.buffer.clear();
        Ok(())
    }

    pub async fn write_str(&mut self, str: &str) -> anyhow::Result<()> {
        self.buffer = str.into();
        self.stream.write_all(&self.buffer).await?;
        self.buffer.clear();
        Ok(())
    }
}

fn disconnected_message(e: anyhow::Error) -> String {
    if let Some(io_error) = e.downcast_ref::<io::Error>() {
        if io_error.kind() == ErrorKind::UnexpectedEof {
            return "disconnected".to_owned();
        }
    }
    format!("{:?}", e)
}

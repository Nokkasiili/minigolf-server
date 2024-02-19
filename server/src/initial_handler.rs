use flume::{Receiver, Sender};
use protocol::{
    client::{ClientToServer, Language, LoginType, TLog, TTLogin, Version},
    common::{PacketNumber, SomeAsTab},
    server::{BasicInfo, Error, ServerToClient, StatusLobbySelect, StatusLogin, VersOk},
};
use rand::Rng;


use crate::listener::Worker;

pub enum InitialHandling {
    Join(NewPlayer),
}
#[derive(Debug)]
pub struct NewPlayer {
    pub network_id: usize,
    pub name: String,
    pub clan: Option<String>,
    pub seed: i32,
    pub language: String,
    pub sent: u32,

    pub received_packets: Receiver<ClientToServer>,
    pub packets_to_send: Sender<ServerToClient>,
}

pub fn add_num(i: &mut u32) -> u32 {
    *i = *i + 1;
    *i
}

fn generate_username() -> String {
    let random_number: u32 = rand::thread_rng().gen_range(0..10000);
    format!("~anonym-{}", random_number)
}

pub async fn handle(worker: &mut Worker) -> anyhow::Result<InitialHandling> {
    /*/
    self.write_str("h 1\n").await?;
    self.write(Io { value: 90000000 }).await?;
    self.write(Crt { value: 250 }).await?;
    self.write(Ctr {}).await?;*/
    let mut sent = 0;

    let seed = protocol::crypt::ConnCipher::get_random_seed();
    worker
        .write_str(&format!("h 1\nc io {}\nc crt 250\nc ctr\n", seed))
        .await?;

    worker.read::<protocol::client::New>().await?;
    let network_id = worker.id_generator().next_id();
    log::debug!("new id {} with {} seed", network_id, seed);
    worker
        .write(protocol::server::Id { value: network_id })
        .await?;
    //let packet = worker.read().await?.command; // skip 1
    let version: Version = worker.read::<Version>().await?;
    if version.version == 35 {
        //worker.write(ServerToClient::VersOk{Vers})?.await?;
        worker
            .write(VersOk {
                packet_number: PacketNumber(0),
            })
            .await?;
    } else {
        worker
            .write(Error {
                packet_number: PacketNumber(0),
                error: protocol::common::DErrorType::VerNotOk,
            })
            .await?;
    }
    let _ = worker.read::<TLog>().await?; //TODO: sometimes packets arrive at different order

    let language = worker.read::<Language>().await?.languge;
    let logintype = worker.read::<LoginType>().await?;

    worker
        .write(StatusLogin {
            packet_number: PacketNumber(add_num(&mut sent)),
            status: SomeAsTab(None),
        })
        .await?;
    let login = worker.read::<TTLogin>().await?;

    /*/
        worker
            .write(StatusLogin {
                packet_number: PacketNumber(sent.add(1)),
                status: SomeAsTab(Some(DLoginStatus::ForbiddenNick)),
            })
            .await?;
    */
    let username = match login.username.0 {
        Some(username) => username,
        None => generate_username(),//TODO
    };

    worker
        .write(BasicInfo {
            packet_number: PacketNumber(add_num(&mut sent)),
            unconfirmed_email: true,
            access_level: 0,
            badword_filter: true,
            guest_chat: false,
        })
        .await?;

    worker
        .write(StatusLobbySelect {
            packet_number: PacketNumber(add_num(&mut sent)),
            lobby: 300,
        })
        .await?;

    Ok(InitialHandling::Join(NewPlayer {
        network_id,
        name: username,
        clan: None,
        language,
        seed,
        sent,
        received_packets: worker.received_packets(),
        packets_to_send: worker.packets_to_send(),
    }))
}

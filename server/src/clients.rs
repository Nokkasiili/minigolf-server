use std::{
    cell::{Cell, RefCell},
    sync::atomic::AtomicU32,
    time::Instant,
};

use flume::{Receiver, Sender};
use protocol::{
    client::ClientToServer,
    common::{DLobbyType, NonEmptyOption, PacketNumber, User},
    server::{LobbyNumberOfUsers, Ping, ServerToClient},
};
use slab::Slab;

use crate::{game::GameId, initial_handler::NewPlayer};

#[derive(Default)]
pub struct Clients {
    arena: Slab<Client>,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClientId(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NetworkId(pub usize);
pub struct Client {
    packets_to_send: Sender<ServerToClient>,
    received_packets: Receiver<ClientToServer>,
    name: String,
    clan: Option<String>,
    lobby: RefCell<Option<DLobbyType>>,
    game: RefCell<Option<GameId>>,
    language: String,
    network_id: NetworkId,
    seed: i32,
    no_challenges: Cell<bool>,
    sent: RefCell<AtomicU32>,
    last_pong: RefCell<Instant>,
    disconnected: Cell<bool>,
    id: Option<ClientId>,
}

impl Client {
    pub fn new(player: NewPlayer) -> Self {
        Self {
            packets_to_send: player.packets_to_send,
            received_packets: player.received_packets,
            name: player.name,
            clan: player.clan,
            language: player.language,
            id: None,
            lobby: RefCell::new(None),
            game: RefCell::new(None),
            network_id: NetworkId(player.network_id),
            seed: player.seed,
            disconnected: Cell::new(false),
            no_challenges: Cell::new(false),
            last_pong: RefCell::new(Instant::now()),
            sent: RefCell::new(AtomicU32::new(player.sent)), // initial handling
        }
    }

    pub fn next_num(&self) -> PacketNumber {
        PacketNumber(
            self.sent
                .borrow_mut()
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        )
    }
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected.get()
    }
    pub fn disconnect(&self) {
        self.disconnected.set(true);
    }
    pub fn set_no_challenges(&self, v: bool) {
        self.no_challenges.set(v);
    }
    pub fn no_challenges(&self) -> bool {
        self.no_challenges.get()
    }

    pub fn id(&self) -> Option<ClientId> {
        self.id
    }

    pub fn lobby_select(&self) -> bool {
        self.game().is_none() && self.lobby().is_none()
    }

    pub fn set_client_id(&mut self, client_id: ClientId) {
        self.id = Some(client_id);
    }

    pub fn game(&self) -> Option<GameId> {
        *self.game.borrow()
    }
    pub fn lobby(&self) -> Option<DLobbyType> {
        *self.lobby.borrow()
    }
    pub fn send_packet(&self, packet: ServerToClient) {
        let _ = self.packets_to_send.try_send(packet);
    }
    pub fn received_packets(&self) -> impl Iterator<Item = ClientToServer> + '_ {
        self.received_packets.try_iter()
    }

    pub fn set_lobby(&self, lobby: Option<DLobbyType>) {
        *self.lobby.borrow_mut() = lobby;
    }
    pub fn set_pong(&self) {
        *self.last_pong.borrow_mut() = Instant::now();
    }
    pub fn last_pong(&self) -> Instant {
        *self.last_pong.borrow()
    }

    pub fn send_ping(&self) {
        self.send_packet(ServerToClient::Ping(Ping {}));
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn set_game(&self, game: Option<GameId>) {
        *self.game.borrow_mut() = game;
    }

    pub fn clan(&self) -> Option<&String> {
        self.clan.as_ref()
    }
    pub fn status_string(&self) -> String {
        //w worm r registered v vip s sherif n no challenges
        let mut s = String::new();

        if let Some('~') = self.name().chars().next() {
            s.push('w');
        } else {
            s.push('r');
        }

        if self.no_challenges() {
            s.push('n');
        }
        s
    }
}

impl Clients {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, client: Client) -> ClientId {
        ClientId(self.arena.insert(client))
    }
    pub fn remove(&mut self, id: ClientId) -> Option<Client> {
        if self.arena.contains(id.0) {
            return Some(self.arena.remove(id.0));
        }
        None
    }

    pub fn get(&self, id: ClientId) -> Option<&Client> {
        self.arena.get(id.0)
    }

    pub fn get_mut(&mut self, id: ClientId) -> Option<&mut Client> {
        self.arena.get_mut(id.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Client> + '_ {
        self.arena.iter().map(|(_i, client)| client)
    }
    pub fn iter_lobby(&self, lobby: DLobbyType) -> impl Iterator<Item = &'_ Client> + '_ {
        self.arena
            .iter()
            .filter(move |(_i, client)| client.lobby() == Some(lobby))
            .map(|(_i, client)| client)
    }
    pub fn lobby_userlist(&self, client_id: ClientId, lobby: DLobbyType) -> Option<Vec<User>> {
        let users: Vec<User> = self
            .arena
            .iter()
            .filter(|(_i, client)| {
                client.lobby() == Some(lobby)
                    && client.id != Some(client_id)
                    && client.game().is_none()
            })
            .map(|(_i, client)| User::from(client))
            .collect();

        Some(users).filter(|users| !users.is_empty())
    }

    pub fn client_from_name(&self, username: &str) -> Option<&Client> {
        self.iter().find(|f| f.name() == username)
    }

    pub fn count_players(&self) -> (i32, i32, i32) {
        let mut solo_count = 0;
        let mut duo_count = 0;
        let mut multi_count = 0;

        for (_i, c) in self.arena.iter() {
            match c.lobby() {
                Some(DLobbyType::Solo) => solo_count += 1,
                Some(DLobbyType::Duo) => duo_count += 1,
                Some(DLobbyType::Multi) => multi_count += 1,
                Some(DLobbyType::SoloIncognito) => solo_count += 1,
                None => (),
            }
        }
        (solo_count, duo_count, multi_count)
    }

    pub fn count_players2(&self, packet_number: PacketNumber) -> LobbyNumberOfUsers {
        let mut single_lobby = 0;
        let mut dual_lobby = 0;
        let mut multi_lobby = 0;
        let mut single_playing = 0;
        let mut dual_playing = 0;
        let mut multi_playing = 0;
        for (_i, c) in self.arena.iter() {
            if c.game().is_some() {
                match c.lobby() {
                    Some(DLobbyType::Solo) => single_playing += 1,
                    Some(DLobbyType::Duo) => dual_playing += 1,
                    Some(DLobbyType::Multi) => multi_playing += 1,
                    Some(DLobbyType::SoloIncognito) => single_playing += 1,
                    None => (),
                }
                continue;
            }
            match c.lobby() {
                Some(DLobbyType::Solo) => single_lobby += 1,
                Some(DLobbyType::Duo) => dual_lobby += 1,
                Some(DLobbyType::Multi) => multi_lobby += 1,
                Some(DLobbyType::SoloIncognito) => single_lobby += 1,
                None => (),
            }
        }
        LobbyNumberOfUsers {
            packet_number: packet_number,
            single_lobby,
            single_playing,
            dual_lobby,
            dual_playing,
            multi_lobby,
            multi_playing,
        }
    }
}

impl From<&Client> for User {
    fn from(val: &Client) -> Self {
        User {
            id_username: format!("3:{}", val.name()),
            value_1: val.status_string(),
            rank: 999,
            lang: val.language.to_string(),
            value_2: NonEmptyOption(None),
            value_3: NonEmptyOption(None),
        }
    }
}

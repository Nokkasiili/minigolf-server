use crate::game::GameServer;
use crate::game::{GameStatus, MinigolfGame};
use crate::handle_packets::game_changed;
use crate::{
    clients::{Client, ClientId, Clients},
    initial_handler::NewPlayer,
    listener::Listener,
    playerid::IdGenerator,
};
use anyhow::Result;
use flume::Receiver;
use protocol::common::DLobbyType;
use protocol::server::GamePart;
use protocol::server::LobbyPart;
use protocol::server::ServerToClient;
use std::time::{Duration, Instant};
pub struct Server {
    pub clients: Clients,
    new_players: Receiver<NewPlayer>,
    pub last_ping: Instant,
}

impl Server {
    pub async fn bind() -> Result<Self> {
        let (new_players_tx, new_players) = flume::bounded(4);
        let id_generator = IdGenerator::new();
        Listener::start(new_players_tx, id_generator.clone()).await?;

        Ok(Self {
            clients: Clients::new(),
            new_players,
            last_ping: Instant::now(),
        })
    }

    pub fn accept_new_players(&mut self) -> Vec<ClientId> {
        let mut clients = Vec::new();
        for player in self.new_players.clone().try_iter() {
            //while let Ok(player) = self.new_players.recv() {
            /*if let Some(old_client) = self.clients.iter().find(|x| x.uuid() == player.uuid) {
                old_client.disconnect("Logged in from another location!");
            }*/
            let id = self.create_client(player);
            clients.push(id);
            self.clients.get_mut(id).unwrap().set_client_id(id);
        }
        clients
    }

    pub fn remove_old_players(&mut self, games: &mut GameServer) {
        let interval = Duration::from_secs(5);

        let clients_to_remove: Vec<_> = self
            .clients
            .iter()
            .filter(|client| {
                client.disconnected()
                    || self.last_ping.duration_since(client.last_pong()) > interval
            })
            .map(|client| client.id())
            .collect();

        for id in clients_to_remove {
            if let Some(id) = id {
                log::debug!("removing {:?}", id);
                let client = self.clients.get(id).unwrap();
                let lobby = client.lobby();
                let game = client.game();
                let name = client.name().to_string();
                self.remove_client(id);

                if let Some(game_id) = game {
                    if let Some(game) = games.get_mut(game_id) {
                        let index = game.get_index(id).unwrap();
                        if game.status() == GameStatus::WaitingPlayers {
                            //TODO own func

                            //Todo turn
                            game.remove_player(index);
                            self.broadcast_game_with(&game, |client| {
                                client.send_packet(ServerToClient::GamePart(GamePart {
                                    packet_number: client.next_num(),
                                    index,
                                    reason: 6,
                                }))
                            });
                            game_changed(self, &games, game_id);
                        } else {
                            self.broadcast_game_with(&game, |client| {
                                client.send_packet(ServerToClient::GamePart(GamePart {
                                    packet_number: client.next_num(),
                                    index,
                                    reason: 4,
                                }))
                            });
                            if let Some(player) = game.players_mut().get_mut(index) {
                                *player = None;
                            }
                        }
                    }
                } else if let Some(lobby) = lobby {
                    self.broadcast_lobby_with(Some(lobby), |c| {
                        c.send_packet(ServerToClient::LobbyPart(LobbyPart {
                            packet_number: c.next_num(),
                            name: name.clone(),
                            reason: protocol::common::JoinLeaveReason::LostConnection,
                        }))
                    })
                }
            }
        }
    }

    /// Removes a client.
    pub fn remove_client(&mut self, id: ClientId) {
        if let Some(client) = self.clients.remove(id) {
            log::debug!("Removed client for {}", client.name());
        }
    }

    fn create_client(&mut self, player: NewPlayer) -> ClientId {
        log::debug!("Creating client for {}", player.name);
        let client = Client::new(player);
        self.clients.insert(client)
    }
    pub fn broadcast_with(&self, mut callback: impl FnMut(&Client)) {
        for client in self.clients.iter() {
            callback(client);
        }
    }
    pub fn broadcast_lobby_with(
        &self,
        lobby: Option<DLobbyType>,
        mut callback: impl FnMut(&Client),
    ) {
        for client in self
            .clients
            .iter()
            .filter(|c| c.lobby() == lobby && c.game().is_none())
        {
            callback(client);
        }
    }
    pub fn broadcast_game_with(&self, game: &MinigolfGame, mut callback: impl FnMut(&Client)) {
        for player in game.players().iter() {
            if let Some(player) = player {
                let client = self.clients.get(player.id);
                if let Some(client) = client {
                    callback(client);
                }
            }
        }
    }

    pub fn broadcast_ping(&mut self) {
        self.broadcast_with(|client| client.send_ping());
        self.last_ping = Instant::now();
    }
}

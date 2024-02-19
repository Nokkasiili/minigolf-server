use anyhow::{bail, Result};
use protocol::{
    client::{self, LobbyChallenge, LobbyCmpt, LobbyCspt},
    common::{Collision, DLobbyType, NonEmptyOption, Scoring, TrackType, WaterEvent, WeightEnd},
    server::{
        Game, GameEnd, GameGameInfo, GameResetVoteSkip, GameStart, GameStartTrack, GameStartTurn,
        LobbyGamelistRemove, LobbyPart, Player, ServerToClient,
    },
};
use slab::Slab;
use std::time::{Duration, Instant};
use std::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    cell::{Cell, Ref, RefCell},
    collections::{HashMap, HashSet},
    f32::consts::E,
    ops::Add,
    sync::atomic::AtomicUsize,
};

use crate::{
    clients::{Client, ClientId},
    server::Server,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameId(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum GameStatus {
    #[default]
    WaitingPlayers,
    WaitingStroke,
    InGame,
    Ended,
}
#[derive(Debug)]
pub struct MinigolfGame {
    game_type: DLobbyType,
    name: Option<String>,
    password: Option<String>,
    permission: i32, //TODO 2 vip 1 reg 0 all
    max_players: usize,
    turn: Cell<usize>,
    cur_track: Cell<usize>,
    num_tracks: usize,
    track_type: TrackType,
    max_strokes: i32,
    time_limit: i32,
    water_event: WaterEvent,
    collision: Collision,
    track_scoring: Scoring,
    track_scoring_weighted_end: WeightEnd,
    status: Cell<GameStatus>,
    network_id: usize,
    last_stroke: Instant,
    players: RefCell<Vec<GamePlayer>>,
}

#[derive(Debug)]
pub struct GamePlayer {
    pub id: ClientId,
    pub in_hole: Cell<bool>,
    pub in_game: Cell<bool>,
    pub want_skip: Cell<bool>,
    pub strokes: Cell<usize>,
    pub has_sent_end_stroke: Cell<bool>,
}

impl MinigolfGame {
    pub fn add_player(&self, id: ClientId) -> Result<()> {
        if self.players.borrow().len() < self.max_players as usize {
            let p = GamePlayer {
                id,
                in_hole: Cell::new(false),
                want_skip: Cell::new(false),
                has_sent_end_stroke: Cell::new(false),
                in_game: Cell::new(true),
                strokes: Cell::new(0),
            };
            self.players.borrow_mut().push(p);
        } else {
            log::error!("Tried to add more players than expected");
            bail!("max players");
        }
        Ok(())
    }

    pub fn remove_player(&mut self, index: usize) {
        self.players.borrow_mut().remove(index);
        //self.players.borrow_mut().retain(|&x| x != id);
    }

    pub fn status(&self) -> GameStatus {
        self.status.get()
    }

    pub fn next_track(&self, server: &Server) {
        let cur_track = self.cur_track.get().add(1);
        if cur_track > self.num_tracks {
            for game_player in self.players().iter() {
                if let Some(client) = server.clients.get(game_player.id) {
                    client.send_packet(ServerToClient::GameEnd(GameEnd {
                        packet_number: client.next_num(),
                        winner: vec![1],
                    }))
                }
            }

            return;
        }

        //TODO HACK
        for game_player in self.players().iter() {
            game_player.in_hole.set(false);
            game_player.want_skip.set(false);
        }
        self.cur_track.set(cur_track);
        let turn = self.get_next_turn();
        if turn.is_none() {
            log::error!("failed to get next turn in next track\n");
            panic!();
        }

        for game_player in self.players().iter() {
            //    game_player.in_hole.set(false);
            //     game_player.want_skip.set(false);
            if let Some(client) = server.clients.get(game_player.id) {
                client.send_packet(ServerToClient::GameResetVoteSkip(GameResetVoteSkip {
                    packet_number: client.next_num(),
                }));
                client.send_packet(ServerToClient::GameStartTrack(track(client, self)));
                client.send_packet(ServerToClient::GameStartTurn(GameStartTurn {
                    packet_number: client.next_num(),
                    index: turn.unwrap(),
                }));
            }
        }
    }

    pub fn is_solo(&self) -> bool {
        self.game_type == DLobbyType::Solo || self.game_type() == DLobbyType::SoloIncognito
    }

    pub fn start(&self, server: &Server) {
        self.status.set(GameStatus::InGame);
        self.cur_track.set(self.cur_track.get().add(1));

        for game_player in self.players().iter() {
            if let Some(client) = server.clients.get(game_player.id) {
                client.send_packet(ServerToClient::GameStart(GameStart {
                    packet_number: client.next_num(),
                }));
                /*  client.send_packet(ServerToClient::GameResetVoteSkip(GameResetVoteSkip {
                    packet_number: client.next_num(),
                }));*/

                client.send_packet(ServerToClient::GameStartTrack(track(client, self)));

                client.send_packet(ServerToClient::GameStartTurn(GameStartTurn {
                    packet_number: client.next_num(),
                    index: self.turn.get(),
                }))
            }
        }
    }

    pub fn get_next_turn(&self) -> Option<usize> {
        for _ in 0..self.players().len() {
            log::debug!("turn:{}", self.turn.get());
            self.turn.set(self.turn.get().add(1) % self.players().len());

            // Check if the player at the new turn index is still in the game and not in a hole
            if self.players.borrow()[self.turn.get()].in_game.get()
                && !self.players.borrow()[self.turn.get()].in_hole.get()
            {
                log::debug!("turn:{}", self.turn.get());
                return Some(self.turn.get());
            }
        }

        None // No valid turn found
    }

    pub fn get_start_track_players_string(&self) -> String {
        let mut status_string = String::new();

        for player in self.players().iter() {
            let status = if player.in_game.get() { 't' } else { 'f' };
            status_string.push(status);
        }

        status_string
    }

    pub fn players(&self) -> std::cell::Ref<'_, Vec<GamePlayer>> {
        self.players.borrow()
    }

    pub fn playing_players(&self) -> usize {
        self.players().iter().filter(|f| f.in_game.get()).count()
    }
    pub fn want_skip(&self) -> bool {
        let skips = self
            .players()
            .iter()
            .filter(|f| f.in_game.get() && (f.want_skip.get() || f.in_hole.get()))
            .count();
        self.playing_players() == skips
    }
    pub fn all_end_strokes(&self) -> bool {
        let skips = self
            .players()
            .iter()
            .filter(|f| f.in_game.get() && f.has_sent_end_stroke.get())
            .count();
        self.playing_players() == skips
    }
    pub fn all_in_hole(&self) -> bool {
        let inhole = self
            .players()
            .iter()
            .filter(|f| f.in_game.get() && f.in_hole.get())
            .count();
        self.playing_players() == inhole
    }

    pub fn max_players(&self) -> usize {
        self.max_players
    }

    pub fn name(&self) -> String {
        self.name.clone().unwrap_or(format!("#{}", self.network_id))
    }

    pub fn get_index(&self, client_id: ClientId) -> Option<usize> {
        for (i, c) in self.players().iter().enumerate() {
            if c.id == client_id {
                return Some(i);
            }
        }
        None
    }

    pub fn game_type(&self) -> DLobbyType {
        self.game_type
    }

    pub fn network_id(&self) -> usize {
        self.network_id
    }

    pub fn turn(&self) -> usize {
        self.turn.get()
    }
}

pub struct GameServer {
    game_rooms: Slab<MinigolfGame>,
    next_network_id: AtomicUsize,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            game_rooms: Slab::new(),
            next_network_id: AtomicUsize::new(1),
        }
    }
    fn next_network_id(&self) -> usize {
        self.next_network_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    pub fn id_from_network_id(&self, id: usize) -> Option<GameId> {
        //self.game_rooms.iter().try_find(|(i, g)| g.network_id == id);

        for (i, g) in self.game_rooms.iter() {
            if g.network_id == id {
                return Some(GameId(i));
            }
        }

        None
    }

    pub fn get(&self, id: GameId) -> Option<&MinigolfGame> {
        self.game_rooms.borrow().get(id.0)
    }
    pub fn get_mut(&mut self, id: GameId) -> Option<&mut MinigolfGame> {
        self.game_rooms.borrow_mut().get_mut(id.0)
    }
    pub fn handle_cmpt(&mut self, client: &Client, packet: &LobbyCmpt) -> GameId {
        let game = MinigolfGame {
            game_type: DLobbyType::Multi,
            name: packet.game_name.0.clone(),
            password: packet.password.0.clone(),
            permission: packet.permission,
            max_players: packet.max_players,
            num_tracks: packet.num_tracks,
            track_type: packet.track_types,
            max_strokes: packet.max_strokes,
            time_limit: packet.time_limit,
            water_event: packet.water_event,
            collision: packet.collision,
            track_scoring: packet.track_scoring,
            track_scoring_weighted_end: packet.track_scoring_weighted_end,
            status: Cell::new(GameStatus::WaitingPlayers),
            network_id: self.next_network_id(),
            turn: Cell::new(0),
            cur_track: Cell::new(0),
            last_stroke: Instant::now(),
            players: RefCell::new(Vec::new()),
        };
        let _ = game.add_player(client.id().unwrap());
        /*server.broadcast_lobby_with(Some(game.game_type), |c| {
            c.send_packet(ServerToClient::LobbyPart(LobbyPart {}))
        });*/
        self.add_game(game)
    }

    pub fn handle_new_challenge(&mut self, packet: &LobbyChallenge, challenger: &str) -> GameId {
        let game = MinigolfGame {
            game_type: DLobbyType::Duo,
            name: Some(packet.challenged.clone()),
            password: Some(challenger.to_string()),
            permission: 0,
            max_players: 2,
            num_tracks: packet.num_tracks,
            track_type: packet.track_types,
            max_strokes: packet.max_strokes,
            time_limit: packet.time_limit,
            water_event: packet.water_event,
            collision: packet.collision,
            track_scoring: packet.track_scoring,
            track_scoring_weighted_end: packet.track_scoring_weighted_end,
            status: Cell::new(GameStatus::WaitingPlayers),
            network_id: self.next_network_id(),
            turn: Cell::new(0),
            cur_track: Cell::new(0),
            last_stroke: Instant::now(),
            players: RefCell::new(Vec::new()),
        };
        self.add_game(game)
    }

    pub fn remove_duo_game(&mut self, name: String) {
        if let Some(index) = self
            .game_rooms
            .iter()
            .position(|(_, g)| g.password == Some(name.clone()) && g.game_type() == DLobbyType::Duo)
        {
            log::debug!("Removing game:{}", index);
            self.game_rooms.remove(index);
        }
    }

    pub fn find_duo_game(&self, challenged: &str, challenger: &str) -> Option<GameId> {
        self.game_rooms
            .iter()
            .find(|(_, g)| {
                g.name.as_deref() == Some(challenged)
                    && g.password.as_deref() == Some(challenger)
                    && g.game_type() == DLobbyType::Duo
            })
            .map(|(i, _)| GameId(i))
    }

    pub fn handle_cspt(&mut self, client: &Client, packet: &LobbyCspt) -> GameId {
        let game = MinigolfGame {
            game_type: DLobbyType::Solo,
            name: None,
            password: None,
            permission: 0,
            max_players: 1,
            turn: Cell::new(0),
            num_tracks: packet.num_tracks,
            track_type: packet.track_type,
            max_strokes: 0,
            time_limit: 0,
            water_event: packet.water_event,
            collision: Collision::Yes,
            track_scoring: Scoring::Score,
            track_scoring_weighted_end: WeightEnd::None,
            status: Cell::new(GameStatus::WaitingPlayers),
            players: RefCell::new(Vec::new()),
            network_id: self.next_network_id(),
            cur_track: Cell::new(0),
            last_stroke: Instant::now(),
        };

        let _ = game.add_player(client.id().unwrap());
        self.add_game(game)
    }

    pub fn add_game(&mut self, game: MinigolfGame) -> GameId {
        let game = self.game_rooms.insert(game);
        log::debug!("Adding game:{}", game);
        GameId(game)
    }

    pub fn handle_rooms(&mut self, server: &Server) {
        let mut rooms_to_remove = HashSet::new();

        for (id, room) in self.game_rooms.iter() {
            if room.status() == GameStatus::WaitingPlayers
                && room.players().len() == room.max_players()
            {
                room.start(server);
                if room.game_type == DLobbyType::Multi {
                    server.broadcast_lobby_with(Some(room.game_type), |c| {
                        c.send_packet(ServerToClient::LobbyGamelistRemove(LobbyGamelistRemove {
                            packet_number: c.next_num(),
                            id: room.network_id,
                        }))
                    })
                }
            }
            if 0 == room.playing_players() {
                rooms_to_remove.insert(id);
            }
            if room.all_end_strokes() {
                //TODO handle scoring
                if let Some(turn) = room.get_next_turn() {
                    server.broadcast_game_with(Some(GameId(id)), |c| {
                        c.send_packet(ServerToClient::GameStartTurn(GameStartTurn {
                            packet_number: c.next_num(),
                            index: turn,
                        }))
                    });
                } else {
                    room.next_track(server);
                }
                for c in room.players().iter() {
                    c.has_sent_end_stroke.set(false);
                }
            }
            if room.want_skip() {
                //TODO handle change scores
                room.next_track(server);
            }

            if room.players().len() == 0 {
                rooms_to_remove.insert(id);
                if room.status() == GameStatus::WaitingPlayers {
                    server.broadcast_lobby_with(Some(room.game_type), |c| {
                        c.send_packet(ServerToClient::LobbyGamelistRemove(LobbyGamelistRemove {
                            packet_number: c.next_num(),
                            id: room.network_id,
                        }))
                    })
                }
            }
        }

        for id in rooms_to_remove {
            self.game_rooms.remove(id);
        }
    }

    pub fn game_list(&self) -> (usize, Option<Vec<Game>>) {
        let games: Vec<Game> = self
            .game_rooms
            .iter()
            .filter(|(_i, g)| g.status() == GameStatus::WaitingPlayers)
            .map(|(_i, game)| Game::from(game))
            .collect();
        (games.len(), Some(games).filter(|games| !games.is_empty()))
    }
}

impl From<&MinigolfGame> for Game {
    fn from(value: &MinigolfGame) -> Self {
        Self {
            id: value.network_id,
            name: value.name(),
            passworded: value.password.is_some(),
            permission: value.permission,
            max_players: value.max_players,
            unused: 1337,
            num_tracks: value.num_tracks,
            track_type: value.track_type,
            max_strokes: value.max_strokes,
            time_limit: value.time_limit,
            water_event: value.water_event,
            collision: value.collision,
            track_scoring: value.track_scoring,
            track_scoring_weighted_end: value.track_scoring_weighted_end,
            num_players: value.players().len(),
        }
    }
}

impl From<&MinigolfGame> for GameGameInfo {
    fn from(value: &MinigolfGame) -> Self {
        GameGameInfo {
            packet_number: protocol::common::PacketNumber(0),
            name: NonEmptyOption(value.name.clone()),
            password: value.password.is_some(),
            permission: value.permission,
            players: value.max_players,
            num_tracks: value.num_tracks,
            track_types: value.track_type,
            max_strokes: value.max_strokes,
            stroke_time: value.time_limit,
            water_event: value.water_event,
            collision: value.collision,
            track_scoring: value.track_scoring,
            track_scoring_weighted_end: value.track_scoring_weighted_end,
            value_2: false,
        }
    }
}

impl From<&client::LobbyChallenge> for MinigolfGame {
    fn from(value: &client::LobbyChallenge) -> Self {
        MinigolfGame {
            game_type: DLobbyType::Duo,
            name: Some(value.challenged.clone()),
            password: None,
            permission: 0,
            max_players: 2,
            turn: Cell::new(0),
            num_tracks: value.num_tracks,
            track_type: value.track_types,
            max_strokes: value.max_strokes,
            time_limit: value.time_limit,
            water_event: value.water_event,
            collision: value.collision,
            track_scoring: value.track_scoring,
            track_scoring_weighted_end: value.track_scoring_weighted_end,
            status: Cell::new(GameStatus::WaitingPlayers),
            network_id: 1337,
            players: RefCell::new(Vec::new()),
            cur_track: Cell::new(0),
            last_stroke: Instant::now(),
        }
    }
}

pub fn track(client: &Client, game: &MinigolfGame) -> GameStartTrack {
    GameStartTrack {
        packet_number: client.next_num(),
        players: game.get_start_track_players_string(),
        seed: 0,
        trackstrings: vec![
                "V 1".to_string(),
                "A Nokkis".to_string(),//
                "N Test".to_string(),//
                "T BA2Q47DCUAECYABA2VCZAGCaAGCbAGC2AB3A36DCBAFEBCWABA2W5GEB3A38D2EB3A46D2EBA2DBABDBACDE40DBWQABA2Q2D5E17DCWI3DE8DCXTDE9DCOA6E14DCWI2DBAMABANABAOABAPAE6DCWTDF2E7D2H2D5E14DBAIABAKAGI10DEG5DC2DBA2NBATDE3DCMA6E11DCE3D4E17DCDCBAMN2ED2H2D5E14D4E17DCD2BAON2E3DCKA6E16D2E17DCDABAPN2ED2H2D5E14DBAKA2DE3DBQAT4DE15DCIA6E20DBIATBA2Q4DCDABJATE11DBPAQH2D5E19DBU2ACDABAGQ3DBAHQBAIQBA2QBRATE12DCJA6E19DBTATBA2QBAFQDBASQD5E10D2H2D5E19D2EBAEQBASQBbASBYASF4E12DCLA6E19D4EB3AD5E10D2H2D5E19D4EBVASD5E12DCNA6E5DCG3DBUASE9D4EHD5E10D2H2D5E19D4EBaASBZAS5E12DCPA6E13DBWMAE4D3EBALQFDBAJQD3E10D2H2D5E13D2E4D4EBAKQ3DCDABU2AE7DB2AQ2DFGD6E13D2E5DBLATCDAI4DBKATB3A4DB2AQE8DECDA2E2CADE12D2E6DBU2ABSAT4DB3AB2AQ4DF3DCT2DCSACQPDCRAECVAFI29DBAR4DBA2Q12D,Ads:A2309B2208C4019".to_string(),
                "S fttf14".to_string(),//
                "C 3,4".to_string(),//
                "I 13942,90651,1,37".to_string(),
                "R 94,12,23,28,28,77,67,49,33,31,279".to_string(),
                "B igo,1283637600000".to_string(),//
                "L igo,1283637600000".to_string(),//
                ],
    }
}

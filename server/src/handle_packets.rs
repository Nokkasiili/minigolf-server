use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    process::id,
    sync::Arc,
};

use protocol::{
    client::{ClientToServer, LobbyChallenge},
    common::{DLobbyType, JoinLeaveReason, NonEmptyOption, SomeAsTab, User},
    server::{
        Game, GameBeginStroke, GameGameInfo, GameJoin, GameOwnInfo, GamePart, GamePlayers, GameSay,
        GameStartTurn, LobbyCFail, LobbyCancel, LobbyGamelistFull, LobbyJoin, LobbyJoinFromGame,
        LobbyNC, LobbyOwnJoin, LobbyPart, LobbySay, LobbySayP, LobbySelectNop, LobbySheriffSay,
        LobbyUsers, Player, ServerToClient, StatusGame, StatusLobby, StatusLobbySelect,
    },
};
use tokio::time::error;

use crate::{
    clients::{Client, ClientId},
    game::{GameId, GameServer, GameStatus, MinigolfGame},
    server::Server,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum OnLobbyJoinFrom {
    Game,
    LobbySelect,
    Lobby,
}

pub fn handle_packets_lobbyselect(
    server: &Server,
    games: &GameServer,
    client: &Client,
    packet: ClientToServer,
) {
    match packet {
        ClientToServer::LobbySelectRnop(_) => {
            let (s, d, m) = server.clients.count_players();
            client.send_packet(ServerToClient::LobbySelectNop(LobbySelectNop {
                packet_number: client.next_num(),
                single: s,
                versus: d,
                multi: m,
            }))
        }
        ClientToServer::LobbySelectCspt(_) => todo!(),
        ClientToServer::LobbySelectQmpt(_) => todo!(),
        ClientToServer::LobbySelectSelect(s) => on_lobby_join(
            &server,
            &client,
            games,
            s.lobby_type,
            OnLobbyJoinFrom::LobbySelect,
        ),
        ClientToServer::Quit(_) => {
            client.disconnect();
        }
        _ => log::error!("{} send wrong packet\n{:?}", client.name(), packet),
    }
}

pub fn handle_packets_single(
    server: &Server,
    games: &mut GameServer,
    client: &Client,
    packet: &ClientToServer,
) {
    match packet {
        ClientToServer::LobbyCspc(_) => todo!(),
        ClientToServer::LobbyTrackSetlist(_) => log::debug!("tracksetlist"),
        _ => {}
    }
}
pub fn handle_packets_dual(
    server: &Server,
    games: &mut GameServer,
    client: &Client,
    packet: &ClientToServer,
) {
    match packet {
        ClientToServer::LobbyNc(nc) => {
            client.set_no_challenges(nc.no_challenges);
            server.broadcast_lobby_with(client.lobby(), |c| {
                if c.id() != client.id() {
                    c.send_packet(ServerToClient::LobbyNC(LobbyNC {
                        packet_number: c.next_num(),
                        name: client.name().to_string(),
                        no_challenges: client.no_challenges(),
                    }))
                }
            });
        }
        ClientToServer::LobbyChallenge(challenge) => {
            if let Some(challenged) = server.clients.client_from_name(&challenge.challenged) {
                let game_id = games.handle_new_challenge(challenge, client.name());
                let game = games.get(game_id).unwrap();
                let _ = game.add_player(client.id().unwrap());

                challenged.send_packet(ServerToClient::LobbyChallenge(
                    protocol::server::LobbyChallenge {
                        packet_number: challenged.next_num(),
                        challenger: client.name().to_string(),
                        num_tracks: challenge.num_tracks,
                        track_types: challenge.track_types,
                        max_strokes: challenge.max_strokes,
                        time_limit: challenge.time_limit,
                        water_event: challenge.water_event,
                        collision: challenge.collision,
                        track_scoring: challenge.track_scoring,
                        track_scoring_weighted_end: challenge.track_scoring_weighted_end,
                    },
                ))
            } else {
                client.send_packet(ServerToClient::LobbyCFail(LobbyCFail {
                    packet_number: client.next_num(),
                    reason: protocol::common::DChallengeFail::NoUser,
                }))
            }
        }
        ClientToServer::LobbyCFail(cfail) => {
            if let Some(other_client) = server.clients.client_from_name(&cfail.name) {
                other_client.send_packet(ServerToClient::LobbyCFail(LobbyCFail {
                    packet_number: other_client.next_num(),
                    reason: cfail.reason,
                }));
                other_client.set_game(None);
                games.remove_duo_game(cfail.name.clone());
            }
        }
        ClientToServer::LobbyCancel(cancel) => {
            if let Some(other_client) = server.clients.client_from_name(&cancel.challenged) {
                other_client.send_packet(ServerToClient::LobbyCancel(LobbyCancel {
                    packet_number: other_client.next_num(),
                }));
                other_client.set_game(None);
                games.remove_duo_game(cancel.challenged.clone());
            }
        }
        ClientToServer::LobbyAccept(accept) => {
            let challenged = client.name();
            let challenger = accept.challenger.clone();

            let other_client = server.clients.client_from_name(&challenger);

            if let Some(other_client) = other_client {
                if let Some(game_id) = games.find_duo_game(challenged, &challenger) {
                    if let Some(game) = games.get(game_id) {
                        let _ = game.add_player(client.id().unwrap());
                        other_client.set_game(Some(game_id));
                        client.set_game(Some(game_id));
                        log::debug!("{} {}", other_client.name(), client.name());
                        game_join(server, client, game);
                        game_join(server, other_client, game);
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn handle_packets_multi(
    server: &Server,
    games: &mut GameServer,
    client: &Client,
    packet: &ClientToServer,
) {
    match packet {
        ClientToServer::LobbyCmpt(packet) => {
            let game_id = games.handle_cmpt(&client, packet);
            client.set_game(Some(game_id));
            let game = games.get(game_id).unwrap();
            server.broadcast_lobby_with(client.lobby(), |c| {
                c.send_packet(ServerToClient::LobbyPart(LobbyPart {
                    packet_number: c.next_num(),
                    name: client.name().to_string(),
                    reason: JoinLeaveReason::CreatedMP(game.name()),
                }))
            });

            server.broadcast_lobby_with(Some(game.game_type()), |c| {
                c.send_packet(ServerToClient::LobbyGamelistAdd(
                    protocol::server::LobbyGamelistAdd {
                        packet_number: c.next_num(),
                        game: Game::from(game),
                    },
                ))
            });

            game_join(server, client, game);
        }
        ClientToServer::LobbyJmpt(packet) => {
            if client.lobby() != Some(DLobbyType::Multi) {
                return;
            }
            if let Some(game_id) = games.id_from_network_id(packet.network_id) {
                if let Some(game) = games.get_mut(game_id) {
                    let _ = game.add_player(client.id().unwrap());
                    client.set_game(Some(game_id));
                    game_join(server, client, game);
                }
                game_changed(server, games, game_id);
            }
        }
        _ => {}
    }
}
pub fn handle_packets_lobby(
    server: &Server,
    games: &mut GameServer,
    client: &Client,
    packet: ClientToServer,
) {
    match packet {
        ClientToServer::LobbyCspt(cspt) => {
            let game_id = games.handle_cspt(client, &cspt);
            let game = games.get(game_id).unwrap();
            client.set_game(Some(game_id));
            game_join(server, client, game);
        }
        ClientToServer::LobbyBack(_) => {
            client.send_packet(ServerToClient::StatusLobbySelect(StatusLobbySelect {
                packet_number: client.next_num(),
                lobby: 300,
            }));
            let lobby = client.lobby();
            client.set_lobby(None);
            server.broadcast_lobby_with(lobby, |c| {
                c.send_packet(ServerToClient::LobbyPart(LobbyPart {
                    packet_number: c.next_num(),
                    name: client.name().to_string(),
                    reason: JoinLeaveReason::LeftLobby,
                }))
            })
        }
        ClientToServer::LobbySelect(s) => on_lobby_join(
            &server,
            &client,
            games,
            s.lobby_type,
            OnLobbyJoinFrom::Lobby,
        ),
        ClientToServer::LobbySay(message_packet) => {
            if let Some(lobby) = client.lobby() {
                for i in server.clients.iter_lobby(lobby) {
                    if i.id() == client.id() {
                        continue;
                    }
                    i.send_packet(ServerToClient::LobbySay(LobbySay {
                        packet_number: i.next_num(),
                        destination: message_packet.lobby_tab.clone(),
                        username: client.name().to_string(),
                        message: message_packet.message.clone(),
                    }))
                }
            }
        }
        ClientToServer::LobbySayP(message_packet) => {
            if let Some(dest) = server.clients.client_from_name(&message_packet.destination) {
                dest.send_packet(ServerToClient::LobbySayP(LobbySayP {
                    packet_number: dest.next_num(),
                    from: client.name().to_string(),
                    message: message_packet.message,
                }));
            }
        }
        ClientToServer::LobbyQuit(_) => client.disconnect(),
        ClientToServer::LobbySelectSelect(s) => on_lobby_join(
            &server,
            &client,
            games,
            s.lobby_type,
            OnLobbyJoinFrom::LobbySelect,
        ),

        _ => {}
    }
}

pub fn handle_packets_game(
    server: &Server,
    games: &mut GameServer,
    client: &Client,
    packet: ClientToServer,
) {
    match packet {
        ClientToServer::GameRate(_) => {
            client.send_packet(ServerToClient::LobbySheriffSay(LobbySheriffSay {
                packet_number: client.next_num(),
                message: "lol".to_string(),
            }));
        }
        ClientToServer::GameStartTurn(_) => todo!(),
        ClientToServer::GameBeginStroke(stroke) => {
            if let Some(game_id) = client.game() {
                if let Some(game) = games.get(game_id) {
                    if let Some(index) = game.get_index(client.id().unwrap()) {
                        if index != game.turn() {
                            log::debug!("{} tried to shoot in a wrong turn", client.name());
                            return;
                        }
                        server.broadcast_game_with(client.game(), |c| {
                            if c.id() != client.id() {
                                c.send_packet(ServerToClient::GameBeginStroke(GameBeginStroke {
                                    packet_number: c.next_num(),
                                    coords: stroke.coords.clone(),
                                    index,
                                }))
                            }
                        });
                    }
                }
            }
        }
        ClientToServer::GameEndStroke(endstroke) => {
            //TODO Check if legit packet

            if let Some(game_id) = client.game() {
                if let Some(game) = games.get(game_id) {
                    if endstroke.index != game.turn() {
                        log::error!("{} ends wrong stroke", client.name());
                        return;
                    }

                    if game.max_players() != endstroke.in_hole.len() {
                        log::error!("{} sent wrong endstroke", client.name());
                        return;
                    }
                    if let Some(index) = game.get_index(client.id().unwrap()) {
                        game.players()[index].has_sent_end_stroke.set(true);
                    }

                    for (i, c) in endstroke.in_hole.chars().enumerate() {
                        if let Some(player) = game.players().get(i) {
                            if player.in_hole.get() == true && c == 'f' {
                                log::error!(
                                    "{} tried to set in_hole back to false (clients are not sync)",
                                    client.name()
                                )
                            }
                            if c == 't' {
                                player.in_hole.set(true);
                            }
                        }
                    }
                }
            }
        }
        ClientToServer::GameSkip(_) => {
            if let Some(game_id) = client.game() {
                if let Some(game) = games.get(game_id) {
                    if game.is_solo() {
                        game.next_track(server);
                    }
                }
            }
        }
        ClientToServer::GameVoteSkip(_) => {
            if let Some(game_id) = client.game() {
                if let Some(game) = games.get(game_id) {
                    if game.status() == GameStatus::WaitingPlayers
                        || game.status() == GameStatus::Ended
                    {
                        return;
                    }
                    if let Some(index) = game.get_index(client.id().unwrap()) {
                        game.players()[index].want_skip.set(true);
                    }

                    if game.want_skip() {
                        game.next_track(server);
                    }
                }
            }
        }
        ClientToServer::GameJoin(packet) => {
            if let Some(game_id) = games.id_from_network_id(packet.id) {
                if let Some(game) = games.get_mut(game_id) {
                    let _ = game.add_player(client.id().unwrap());
                    let index = game.get_index(client.id().unwrap()).unwrap();
                    for game_player in game.players().iter() {
                        if Some(game_player.id) == client.id() {
                            continue;
                        }
                        let client = server.clients.get(game_player.id).unwrap();
                        client.send_packet(ServerToClient::GameJoin(GameJoin {
                            packet_number: client.next_num(),
                            index,
                            name: client.name().to_string(),
                            clan: NonEmptyOption(client.clan().cloned()),
                        }));
                    }
                    client.set_game(Some(game_id));
                }
                game_changed(server, games, game_id);
            }
        }
        ClientToServer::GameBack(_) => {
            if let Some(game) = games.get_mut(client.game().unwrap()) {
                let index = game.get_index(client.id().unwrap()).unwrap();

                for game_player in game.players().iter() {
                    //TODO own func
                    if let Some(client) = server.clients.get(game_player.id) {
                        let reason = match game.status() {
                            GameStatus::WaitingPlayers => 6,
                            _ => 4,
                        };
                        client.send_packet(ServerToClient::GamePart(GamePart {
                            packet_number: client.next_num(),
                            index,
                            reason: reason,
                        }))
                    }
                }
                if game.status() == GameStatus::WaitingPlayers {
                    game.remove_player(index);
                    game_changed(server, &games, client.game().unwrap());
                } else {
                    game.players().get(index).unwrap().in_game.set(false);
                }
                client.set_game(None);

                on_lobby_join(
                    server,
                    client,
                    games,
                    client.lobby().unwrap(),
                    OnLobbyJoinFrom::Game,
                );
            }
        }
        ClientToServer::GameSay(packet) => {
            if let Some(game) = games.get(client.game().unwrap()) {
                let index = game.get_index(client.id().unwrap()).unwrap();
                for game_player in game.players().iter() {
                    if game_player.id == client.id().unwrap() {
                        continue;
                    }

                    if let Some(client) = server.clients.get(game_player.id) {
                        client.send_packet(ServerToClient::GameSay(GameSay {
                            packet_number: client.next_num(),
                            index,
                            message: packet.message.clone(),
                        }))
                    }
                }
            }
        }

        ClientToServer::GameNewGame(_) => {}
        _ => {}
    }
}

pub fn on_lobby_join(
    server: &Server,
    client: &Client,
    games: &GameServer,
    lobby_type: DLobbyType,
    from: OnLobbyJoinFrom,
) {
    log::debug!("{} joining {}", client.name(), lobby_type);
    client.send_packet(ServerToClient::StatusLobby(StatusLobby {
        packet_number: client.next_num(),
        lobby: lobby_type,
    }));
    let last_lobby = client.lobby();
    client.set_lobby(Some(lobby_type));
    if lobby_type != DLobbyType::SoloIncognito {
        client.send_packet(ServerToClient::LobbyNumberOfUsers(
            server.clients.count_players2(client.next_num()),
        ));

        client.send_packet(ServerToClient::LobbyUsers(LobbyUsers {
            packet_number: client.next_num(),
            users: SomeAsTab(
                server
                    .clients
                    .lobby_userlist(client.id().unwrap(), lobby_type),
            ),
        }));
        client.send_packet(ServerToClient::LobbyOwnJoin(LobbyOwnJoin {
            packet_number: client.next_num(),
            own_info: User::from(client),
        }));
        server.broadcast_with(|c| {
            if c.id() != client.id() {
                if c.lobby() == client.lobby() {
                    if from == OnLobbyJoinFrom::Game {
                        c.send_packet(ServerToClient::LobbyJoinFromGame(LobbyJoinFromGame {
                            packet_number: c.next_num(),
                            user: User::from(client),
                        }))
                    } else {
                        c.send_packet(ServerToClient::LobbyJoin(LobbyJoin {
                            packet_number: c.next_num(),
                            user: User::from(client),
                        }))
                    }
                }
                if from == OnLobbyJoinFrom::Lobby && last_lobby.is_some() && last_lobby == c.lobby()
                {
                    c.send_packet(ServerToClient::LobbyPart(LobbyPart {
                        packet_number: c.next_num(),
                        name: client.name().to_string(),
                        reason: JoinLeaveReason::LeftLobby,
                    }))
                }
            }
        });
    }

    if lobby_type == DLobbyType::Multi {
        let (len, gamelist) = games.game_list();
        client.send_packet(ServerToClient::LobbyGamelistFull(LobbyGamelistFull {
            packet_number: client.next_num(),
            len: len,
            games: gamelist,
        }))
    }
}
pub fn game_changed(server: &Server, games: &GameServer, game_id: GameId) {
    if let Some(game) = games.get(game_id) {
        server.broadcast_lobby_with(Some(game.game_type()), |c| {
            c.send_packet(ServerToClient::LobbyGamelistChange(
                protocol::server::LobbyGamelistChange {
                    packet_number: c.next_num(),
                    game: Game::from(game),
                },
            ))
        });
    }
}

pub fn game_join(server: &Server, client: &Client, game: &MinigolfGame) {
    client.send_packet(ServerToClient::StatusGame(StatusGame {
        packet_number: client.next_num(),
    }));

    let index = game.get_index(client.id().unwrap()).unwrap();
    let mut gameinfo = GameGameInfo::from(game);
    gameinfo.packet_number = client.next_num();

    client.send_packet(ServerToClient::GameGameInfo(gameinfo));

    let mut players = Vec::new();
    for game_player in game.players().iter() {
        if Some(game_player.id) == client.id() {
            continue;
        }
        if let Some(other_client) = server.clients.get(game_player.id) {
            if let Some(player_index) = game.get_index(other_client.id().unwrap()) {
                players.push(Player {
                    index: player_index,
                    name: other_client.name().to_string(),
                    clan: NonEmptyOption(other_client.clan().cloned()),
                });

                if game.game_type() == DLobbyType::Multi {
                    other_client.send_packet(ServerToClient::GameJoin(GameJoin {
                        packet_number: other_client.next_num(),
                        index,
                        name: client.name().to_string(),
                        clan: NonEmptyOption(client.clan().cloned()),
                    }));
                }
            }
        }
    }

    let players = Some(players).filter(|players| !players.is_empty());
    client.send_packet(ServerToClient::GamePlayers(GamePlayers {
        packet_number: client.next_num(),
        players: SomeAsTab(players),
    }));
    client.send_packet(ServerToClient::GameOwnInfo(GameOwnInfo {
        packet_number: client.next_num(),
        index,
        name: client.name().to_string(),
        clan: NonEmptyOption(client.clan().cloned()),
    }));
}

pub fn send_userlist(server: &Server, lobby: DLobbyType) {
    for client in server.clients.iter_lobby(lobby) {
        client.send_packet(ServerToClient::LobbyUsers(LobbyUsers {
            packet_number: client.next_num(),
            users: SomeAsTab(server.clients.lobby_userlist(client.id().unwrap(), lobby)),
        }));
        client.send_packet(ServerToClient::LobbyOwnJoin(LobbyOwnJoin {
            packet_number: client.next_num(),
            own_info: User {
                id_username: format!("3:{}", client.name()),
                value_1: "r".to_owned(),
                rank: 999,
                lang: client.language().to_owned(),
                value_2: NonEmptyOption(None),
                value_3: NonEmptyOption(None),
            },
        }));
    }
}

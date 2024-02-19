use std::time::{Duration, Instant};

use crate::server::Server;
use anyhow::Result;
use game::GameServer;
use protocol::client::{ClientToServer, Pong};
use tickloop::TickLoop;

mod clients;
//mod crypt;
mod codec;
mod define;
mod filter;
mod game;
mod handle_packets;
mod initial_handler;
mod listener;
mod playerid;
mod server;
mod tickloop;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let mut server = Server::bind().await?;
    let mut games = GameServer::new();

    let interval = Duration::from_secs(5);

    let tickloop = TickLoop::new(move || {
        server.accept_new_players();
        server.remove_old_players(&mut games);
        games.handle_rooms(&server);
        for client in server.clients.iter() {
            for packet in client.received_packets() {
                log::debug!("handling: {:?}", packet);
                if matches!(packet, ClientToServer::Pong(Pong {})) {
                    client.set_pong();
                } else if client.lobby_select() {
                    handle_packets::handle_packets_lobbyselect(
                        &server, &mut games, &client, packet,
                    );
                } else if client.lobby().is_some() && client.game().is_none() {
                    match client.lobby().unwrap() {
                        protocol::common::DLobbyType::Solo
                        | protocol::common::DLobbyType::SoloIncognito => {
                            handle_packets::handle_packets_single(
                                &server, &mut games, client, &packet,
                            )
                        }
                        protocol::common::DLobbyType::Duo => handle_packets::handle_packets_dual(
                            &server, &mut games, client, &packet,
                        ),
                        protocol::common::DLobbyType::Multi => {
                            handle_packets::handle_packets_multi(
                                &server, &mut games, client, &packet,
                            )
                        }
                    }
                    handle_packets::handle_packets_lobby(&server, &mut games, &client, packet);
                } else {
                    //we should be in game
                    handle_packets::handle_packets_game(&server, &mut games, &client, packet);
                }
            }
        }

        if server.last_ping + interval < Instant::now() {
            server.broadcast_ping();
        }
        false
    });
    tickloop.run();

    Ok(())
}

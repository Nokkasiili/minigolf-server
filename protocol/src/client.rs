use crate::common::{
    Collision, DLobbyType, DLoginType, NonEmptyOption, NoneAsTab, Parse, Scoring, TrackType,
    WaterEvent, WeightEnd,
};
use crate::common::{DChallengeFail, PlayerInfo};
use parsemacro::Parse as ParseD;

use crate::common::Packet;
use crate::common::PacketNumber;
use nom::IResult;

//D

#[derive(Debug, ParseD)]
#[parse(tag = "version")]
pub struct Version {
    pub packet_number: PacketNumber,
    pub version: i32,
}

#[derive(Debug, ParseD)]
#[parse(tag = "language")]
pub struct Language {
    pub packet_number: PacketNumber,
    pub languge: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "logintype")]
pub struct LoginType {
    pub packet_number: PacketNumber,
    pub login_type: DLoginType,
}

#[derive(Debug, ParseD)]
#[parse(tag = "login")]
pub struct Login {
    pub packet_number: PacketNumber,
    pub session: Option<i32>,
}
#[derive(Debug, ParseD)]
#[parse(tag = "ttlogin")]
pub struct TTLogin {
    pub packet_number: PacketNumber,
    pub username: NoneAsTab<String>,
    pub password: NoneAsTab<String>,
}
//D
#[derive(Debug, ParseD)]
#[parse(tag = "quit")]
pub struct Quit {
    pub packet_number: PacketNumber,
}

// D LOBBYSELECT

#[derive(Debug, ParseD)]
#[parse(tag = "lobbyselect\trnop")] //request number of players
pub struct LobbySelectRnop {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobbyselect\tcspt")]
pub struct LobbySelectCspt {
    pub packet_number: PacketNumber,
    pub num_tracks: usize,
    pub track_type: TrackType,
    pub water_event: WaterEvent,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobbyselect\tqmpt")]
pub struct LobbySelectQmpt {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobbyselect\tselect")]
pub struct LobbySelectSelect {
    pub packet_number: PacketNumber,
    pub lobby_type: DLobbyType,
}
//LOBBY

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tback")]
pub struct LobbyBack {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tselect")]
pub struct LobbySelect {
    pub packet_number: PacketNumber,
    pub lobby_type: DLobbyType, //DLobbyType,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\ttracksetlist")]
pub struct LobbyTrackSetlist {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tcspt")]
pub struct LobbyCspt {
    pub packet_number: PacketNumber,
    pub num_tracks: usize,
    pub track_type: TrackType,
    pub water_event: WaterEvent,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tcmpt")]
pub struct LobbyCmpt {
    pub packet_number: PacketNumber,
    pub game_name: NonEmptyOption<String>,
    pub password: NonEmptyOption<String>,
    pub permission: i32, //TODO
    pub max_players: usize,
    pub num_tracks: usize,
    pub track_types: TrackType,
    pub max_strokes: i32,
    pub time_limit: i32,
    pub water_event: WaterEvent,
    pub collision: Collision,
    pub track_scoring: Scoring,
    pub track_scoring_weighted_end: WeightEnd,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tsay")]
pub struct LobbySay {
    pub packet_number: PacketNumber,
    pub lobby_tab: String,
    pub message: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tnc")]
pub struct LobbyNc {
    pub packet_number: PacketNumber,
    pub no_challenges: bool,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tcfail")]
pub struct LobbyCFail {
    pub packet_number: PacketNumber,
    pub name: String,
    pub reason: DChallengeFail,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tsayp")]
pub struct LobbySayP {
    pub packet_number: PacketNumber,
    pub destination: String,
    pub message: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tjmpt")]
pub struct LobbyJmpt {
    pub packet_number: PacketNumber,
    pub network_id: usize,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tcspc")]
pub struct LobbyCspc {
    pub packet_number: PacketNumber,
    pub network_id: usize,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tcancel")]
pub struct LobbyCancel {
    pub packet_number: PacketNumber,
    pub challenged: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\taccept")]
pub struct LobbyAccept {
    pub packet_number: PacketNumber,
    pub challenger: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tchallenge")]
pub struct LobbyChallenge {
    pub packet_number: PacketNumber,
    pub challenged: String,
    pub num_tracks: usize,
    pub track_types: TrackType,
    pub max_strokes: i32,
    pub time_limit: i32,
    pub water_event: WaterEvent,
    pub collision: Collision,
    pub track_scoring: Scoring,
    pub track_scoring_weighted_end: WeightEnd,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tquit")]
pub struct LobbyQuit {
    pub packet_number: PacketNumber,
}
//D GAME

#[derive(Debug, ParseD)]
#[parse(tag = "game\trate")]
pub struct GameRate {
    pub packet_number: PacketNumber,
    pub track_num: u8,
    pub rating: u8,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tstartturn")]
pub struct GameStartTurn {
    pub packet_number: PacketNumber,
    pub id: i32,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tbeginstroke")]
pub struct GameBeginStroke {
    pub packet_number: PacketNumber,
    pub coords: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tendstroke")]
pub struct GameEndStroke {
    pub packet_number: PacketNumber,
    pub index: usize,
    //pub in_hole: PlayerInfo,
    pub in_hole: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tskip")]
pub struct GameSkip {
    pub packet_number: PacketNumber,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tnewgame")]
pub struct GameNewGame {
    pub packet_number: PacketNumber,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tbacktoprivate")]
pub struct GameBackToPrivate {
    pub packet_number: PacketNumber,
    pub value_1: i32,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\trejectaccept")]
pub struct GameRejectAccept {
    pub packet_number: PacketNumber,
    pub track: i32,
    pub value: bool,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tvoteskip")]
pub struct GameVoteSkip {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tjoin")]
pub struct GameJoin {
    pub packet_number: PacketNumber,
    pub id: usize,
    pub username: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tsay")]
pub struct GameSay {
    pub packet_number: PacketNumber,
    pub message: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tback")]
pub struct GameBack {
    pub packet_number: PacketNumber,
}
// S

#[derive(Debug, ParseD)]
#[parse(tag = "s tlog")]
pub struct TLog {
    pub count: i32,
    pub id: String,
    pub str: String,
    //pub log: Vec<String>,
}

// C
#[derive(Debug, ParseD)]
#[parse(tag = "c new", space = true)]
pub struct New {}

#[derive(Debug, ParseD)]
#[parse(tag = "c old", space = true)]
pub struct Old {
    pub id: i32,
}
#[derive(Debug, ParseD)]
#[parse(tag = "c pong")]
pub struct Pong {}

#[derive(Debug, ParseD)]
pub enum ClientToServer {
    Version(Version),
    Language(Language),
    LoginType(LoginType),
    Login(Login),
    TTLogin(TTLogin),
    LobbySelectRnop(LobbySelectRnop),
    LobbySelectCspt(LobbySelectCspt),
    LobbySelectQmpt(LobbySelectQmpt),
    LobbySelectSelect(LobbySelectSelect),
    LobbySayP(LobbySayP),
    LobbyAccept(LobbyAccept),
    LobbyCancel(LobbyCancel),
    LobbyCspt(LobbyCspt),
    LobbyBack(LobbyBack),
    LobbySelect(LobbySelect),
    LobbyChallenge(LobbyChallenge),
    LobbyTrackSetlist(LobbyTrackSetlist),
    LobbyCmpt(LobbyCmpt),
    LobbySay(LobbySay),
    LobbyCFail(LobbyCFail),
    LobbyNc(LobbyNc),
    LobbyJmpt(LobbyJmpt),
    LobbyCspc(LobbyCspc),
    LobbyQuit(LobbyQuit),
    GameRate(GameRate),
    GameStartTurn(GameStartTurn),
    GameBeginStroke(GameBeginStroke),
    GameEndStroke(GameEndStroke),
    GameBackToPrivate(GameBackToPrivate),
    GameRejectAccept(GameRejectAccept),
    GameSkip(GameSkip),
    GameNewGame(GameNewGame),
    GameVoteSkip(GameVoteSkip),
    GameJoin(GameJoin),
    GameBack(GameBack),
    GameSay(GameSay),
    Quit(Quit),
    TLog(TLog),
    New(New),
    Old(Old),
    Pong(Pong),
}

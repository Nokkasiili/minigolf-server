use crate::common::Collision;
use crate::common::DChallengeFail;
use crate::common::DErrorType;
use crate::common::DLobbyType;
use crate::common::DLoginStatus;
use crate::common::Difficulty;
use crate::common::JoinLeaveReason;
use crate::common::KickStyle;
use crate::common::NonEmptyOption;
use crate::common::Packet;
use crate::common::PacketNumber;
use crate::common::Parse;
use crate::common::Scoring;
use crate::common::SomeAsTab;
use crate::common::TrackType;
use crate::common::User;
use crate::common::WaterEvent;
use crate::common::WeightEnd;
use nom::IResult;
use parsemacro::Parse as ParseD;

// H
#[derive(Debug, ParseD)]
#[parse(tag = "h", space = true)]
pub struct H {
    pub value: i32,
}

// S

#[derive(Debug, ParseD)]
#[parse(tag = "s")]
pub struct Version {
    pub value: String,
}
// P

#[derive(Debug, ParseD)]
#[parse(tag = "p kickban")]
pub struct KickBan {
    pub value: KickStyle,
}
// C

#[derive(Debug, ParseD)]
#[parse(tag = "c io", space = true)]
pub struct Io {
    pub seed: i32,
}

#[derive(Debug, ParseD)]
#[parse(tag = "c crt", space = true)]
pub struct Crt {
    pub value: i32,
}

#[derive(Debug, ParseD)]
#[parse(tag = "c ctr", space = true)]
pub struct Ctr {}

#[derive(Debug, ParseD)]
#[parse(tag = "c id", space = true)]
pub struct Id {
    pub value: usize,
}

#[derive(Debug, ParseD)]
#[parse(tag = "c ping", space = true)]
pub struct Ping {}

#[derive(Debug, ParseD)]
#[parse(tag = "c rcok", space = true)]
pub struct Rcok {}

#[derive(Debug, ParseD)]
#[parse(tag = "c rcf", space = true)]
pub struct Rcf {}

//D

#[derive(Debug, ParseD)]
#[parse(tag = "versok")]
pub struct VersOk {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "error")]
pub struct Error {
    pub packet_number: PacketNumber,
    pub error: DErrorType,
}

#[derive(Debug, ParseD)]
#[parse(tag = "basicinfo")]
pub struct BasicInfo {
    pub packet_number: PacketNumber,
    pub unconfirmed_email: bool,
    pub access_level: i32,
    pub badword_filter: bool,
    pub guest_chat: bool,
}

#[derive(Debug, ParseD)]
#[parse(tag = "broadcast")]
pub struct Broadcast {
    pub packet_number: PacketNumber,
    pub broadcast: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tgameinfo")]
pub struct GameGameInfo {
    /*
    <= "d 9 game\tgameinfo\t-\tf\t13\t3\t10\t1\t20\t60\t0\t1\t0\t0\tf\n"
    [[info	-	f	13	3	10	1	20	60	0	1	0	0	f
    ]]*/
    pub packet_number: PacketNumber,
    pub name: NonEmptyOption<String>,
    pub password: bool,
    pub permission: i32,
    pub players: usize,
    pub num_tracks: usize,
    pub track_types: TrackType,
    pub max_strokes: i32,
    pub stroke_time: i32,
    pub water_event: WaterEvent,
    pub collision: Collision,
    pub track_scoring: Scoring,
    pub track_scoring_weighted_end: WeightEnd,
    pub value_2: bool,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tplayers", opt_f = true)]
pub struct GamePlayers {
    pub packet_number: PacketNumber,
    pub players: SomeAsTab<Vec<Player>>,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tend")]
pub struct GameEnd {
    pub packet_number: PacketNumber,
    pub winner: Vec<i32>, // 1, -1 , 1, -1  //first and third are winners
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\towninfo")]
pub struct GameOwnInfo {
    pub packet_number: PacketNumber,
    pub index: usize,
    pub name: String,
    pub clan: NonEmptyOption<String>,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tscoringmulti")]
pub struct GameScoringMulti {
    pub packet_number: PacketNumber,
    pub scoring_multipliers: Vec<i32>,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tcr")]
pub struct GameCr {
    pub packet_number: PacketNumber,
    pub token: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tchangescore")]
pub struct GameChangeScore {
    pub packet_number: PacketNumber,
    pub scores: Vec<i32>,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tvoteskip")]
pub struct GameVoteSkip {
    pub packet_number: PacketNumber,
    pub index: usize,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\trfng")] // ready for newgame
pub struct GameRfng {
    pub packet_number: PacketNumber,
    pub index: usize,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tresetvoteskip")]
pub struct GameResetVoteSkip {
    pub packet_number: PacketNumber,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tstarttrack")]
pub struct GameStartTrack {
    pub packet_number: PacketNumber,
    pub players: String, // t for every playing player or testmode ttm1, ttm2
    //there could be 1 arg if testmode
    pub seed: i32,
    pub trackstrings: Vec<String>,
    /*trackstring_2: String,
    trackstring_3: String,
    trackstring_4: String,
    trackstring_5: String,
    trackstring_6: String,
    trackstring_7: String,*/
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tgame")]
pub struct GameGame {
    pub packet_number: PacketNumber,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tstartturn")]
pub struct GameStartTurn {
    pub packet_number: PacketNumber,
    pub index: usize,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tstart")]
pub struct GameStart {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "game\tsay")]
pub struct GameSay {
    pub packet_number: PacketNumber,
    pub index: usize,
    pub message: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tpart")]
pub struct GamePart {
    pub packet_number: PacketNumber,
    pub index: usize,
    pub reason: usize,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tjoin")]
pub struct GameJoin {
    pub packet_number: PacketNumber,
    pub index: usize,
    pub name: String,
    pub clan: NonEmptyOption<String>,
}
#[derive(Debug, ParseD)]
#[parse(tag = "game\tbeginstroke")]
pub struct GameBeginStroke {
    pub packet_number: PacketNumber,
    pub index: usize,
    pub coords: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "status\tlogin", notab = true)]
pub struct StatusLogin {
    pub packet_number: PacketNumber,
    pub status: SomeAsTab<DLoginStatus>,
}

#[derive(Debug, ParseD)]
#[parse(tag = "status\tgame")]
pub struct StatusGame {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "status\tlobby")]
pub struct StatusLobby {
    pub packet_number: PacketNumber,
    pub lobby: DLobbyType,
}

#[derive(Debug, ParseD)]
#[parse(tag = "status\tlobbyselect")]
pub struct StatusLobbySelect {
    pub packet_number: PacketNumber,
    pub lobby: i32,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\ttracksetlist")]
pub struct LobbyTrackSetlist {
    pub packet_number: PacketNumber,
    pub setlist: Option<Vec<Tracklist>>,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tnumberofusers")]
pub struct LobbyNumberOfUsers {
    pub packet_number: PacketNumber,
    pub single_lobby: i32,
    pub single_playing: i32,
    pub dual_lobby: i32,
    pub dual_playing: i32,
    pub multi_lobby: i32,
    pub multi_playing: i32,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\townjoin")]
pub struct LobbyOwnJoin {
    pub packet_number: PacketNumber,
    pub own_info: User,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tjoinfromgame")]
pub struct LobbyJoinFromGame {
    pub packet_number: PacketNumber,
    pub user: User,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tjoin")]
pub struct LobbyJoin {
    pub packet_number: PacketNumber,
    pub user: User,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tcfail")]
pub struct LobbyCFail {
    pub packet_number: PacketNumber,
    pub reason: DChallengeFail,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tafail")]
pub struct LobbyAFail {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tcancel")]
pub struct LobbyCancel {
    pub packet_number: PacketNumber,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tchallenge")]
pub struct LobbyChallenge {
    pub packet_number: PacketNumber,
    pub challenger: String,
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
#[parse(tag = "lobby\tnc")]
pub struct LobbyNC {
    pub packet_number: PacketNumber,
    pub name: String,
    pub no_challenges: bool,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tsherifsay")]
pub struct LobbySheriffSay {
    pub packet_number: PacketNumber,
    pub message: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tsay")]
pub struct LobbySay {
    pub packet_number: PacketNumber,
    pub destination: String,
    pub username: String,
    pub message: String,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tsayp")]
pub struct LobbySayP {
    pub packet_number: PacketNumber,
    pub from: String,
    pub message: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tgsn")] //game starts duo
pub struct LobbyGsn {
    pub packet_number: PacketNumber,
    pub challenger: String,
    pub challenged: String,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tusers", opt_f = true)]
pub struct LobbyUsers {
    pub packet_number: PacketNumber,
    //"d 7 lobby\tusers\t3:~anonym-2893^wn^-1^de_DE^-^-\t3:Benny11112222^r^10^de_DE^-^-\t3:Jomppppa^rn^146^fi_FI^-^-"
    //pub value_1: Option<String>, //TODO
    pub users: SomeAsTab<Vec<User>>,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tpart")]
pub struct LobbyPart {
    pub packet_number: PacketNumber,
    // "d 17 lobby\tpart\tzocker666\t2\t#1583093"
    pub name: String,
    pub reason: JoinLeaveReason,
    //TODO
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tgamelist\tfull")]
pub struct LobbyGamelistFull {
    pub packet_number: PacketNumber,
    pub len: usize,
    pub games: Option<Vec<Game>>, //TODO new type that ads \t for end and before
}

#[derive(Debug, ParseD)]
#[parse(tag = "", notab = true, notag = true)]
pub struct Tracklist {
    name: String,
    difficulty: Difficulty,
    tracks: i32,
    all_time_best_name: String,
    all_time_best_strokes: i32,

    month_best_name: String,
    month_best_strokes: i32,

    week_best_name: String,
    week_best_strokes: i32,

    day_best_name: String,
    day_best_strokes: i32,
}
#[derive(Debug, ParseD)]
#[parse(tag = "", notab = true, notag = true)]
pub struct Game {
    pub id: usize,
    pub name: String,
    pub passworded: bool,
    pub permission: i32, //TODO 2 vip 1 reg 0 all
    pub max_players: usize,
    pub unused: i32,
    pub num_tracks: usize,
    pub track_type: TrackType,
    pub max_strokes: i32,
    pub time_limit: i32,
    pub water_event: WaterEvent,
    pub collision: Collision,
    pub track_scoring: Scoring,
    pub track_scoring_weighted_end: WeightEnd,
    pub num_players: usize,
    //pub fucked: FuckedType<i32>,
}

#[derive(Debug, ParseD)]
#[parse(tag = "", notab = true, notag = true)]
pub struct Player {
    pub index: usize,
    pub name: String,
    pub clan: NonEmptyOption<String>,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tgamelist\tremove")]
pub struct LobbyGamelistRemove {
    pub packet_number: PacketNumber,
    pub id: usize,
}
#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tgamelist\tchange")]
pub struct LobbyGamelistChange {
    pub packet_number: PacketNumber,
    pub game: Game,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobby\tgamelist\tadd")]
pub struct LobbyGamelistAdd {
    pub packet_number: PacketNumber,
    pub game: Game,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobbyselect\tnop")] // Number of players
pub struct LobbySelectNop {
    pub packet_number: PacketNumber,
    pub single: i32,
    pub versus: i32,
    pub multi: i32,
}

#[derive(Debug, ParseD)]
#[parse(tag = "lobbyselect\tlobby")]
pub struct LobbySelectLobby {
    pub packet_number: PacketNumber,
    pub value: i32,
}

#[derive(Debug, ParseD)]
pub enum ServerToClient {
    GameGameInfo(GameGameInfo), //behind gamegame
    H(H),
    Version(Version),
    KickBan(KickBan),
    Io(Io),
    Crt(Crt),
    Ctr(Ctr),
    Id(Id),
    Ping(Ping),
    Rcok(Rcok),
    Rcf(Rcf),
    VersOk(VersOk),
    Error(Error),
    BasicInfo(BasicInfo),
    Broadcast(Broadcast),
    GamePlayers(GamePlayers),
    GameOwnInfo(GameOwnInfo),
    GameScoringMulti(GameScoringMulti),
    GameCr(GameCr),
    GameChangeScore(GameChangeScore),
    GameVoteSkip(GameVoteSkip),
    GamePart(GamePart),
    GameRfng(GameRfng),
    GameResetVoteSkip(GameResetVoteSkip),
    GameEnd(GameEnd),
    GameSay(GameSay),
    GameJoin(GameJoin),
    GameStartTrack(GameStartTrack),
    GameStartTurn(GameStartTurn),
    GameBeginStroke(GameBeginStroke),
    StatusLogin(StatusLogin),
    StatusGame(StatusGame),
    StatusLobby(StatusLobby),
    StatusLobbySelect(StatusLobbySelect),
    LobbyTrackSetlist(LobbyTrackSetlist),
    LobbyNumberOfUsers(LobbyNumberOfUsers),
    LobbyOwnJoin(LobbyOwnJoin),
    LobbyJoinFromGame(LobbyJoinFromGame),
    LobbyJoin(LobbyJoin),
    LobbyCFail(LobbyCFail),
    LobbyAFail(LobbyAFail),
    LobbyCancel(LobbyCancel),
    LobbyNC(LobbyNC),
    LobbyChallenge(LobbyChallenge),
    LobbySheriffSay(LobbySheriffSay),
    LobbySay(LobbySay),
    LobbySayP(LobbySayP),
    LobbyGsn(LobbyGsn),
    LobbyUsers(LobbyUsers),
    LobbyPart(LobbyPart),
    LobbyGamelistFull(LobbyGamelistFull),
    /*Tracklist(Tracklist),
    Game(Game),
    Player(Player),*/
    LobbyGamelistRemove(LobbyGamelistRemove),
    LobbyGamelistChange(LobbyGamelistChange),
    LobbyGamelistAdd(LobbyGamelistAdd),
    LobbySelectNop(LobbySelectNop),
    LobbySelectLobby(LobbySelectLobby),
    GameGame(GameGame),
    GameStart(GameStart),
}

#[cfg(test)]
mod tests {

    use std::assert_matches::assert_matches;

    use super::GameGameInfo;
    use crate::{common::Parse, server::LobbySayP};
    #[test]
    fn gameinfo_test() {
        let str = "d 9 game\tgameinfo\t-\tf\t13\t3\t10\t1\t20\t60\t0\t1\t0\t0\tf\n";
        assert_matches!(GameGameInfo::parse(&str).unwrap().1, GameGameInfo { .. });
        assert_eq!(GameGameInfo::parse(&str).unwrap().1.as_string(), str);
    }

    #[test]
    fn chat_test() {
        let input = "d 5 lobby	sayp	Nokkasiili	lol lol lol\n";
        assert_eq!(LobbySayP::parse(input).unwrap().1.as_string(), input);
    }
}

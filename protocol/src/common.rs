use core::fmt;
use std::fmt::write;

use casey::lower;
use derive_more::Display;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::character::complete::anychar;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::error::make_error;
use nom::error::ErrorKind;

use nom::multi::separated_list1;
use nom::Err as NomErr;
use nom::IResult;
use std::fmt::Display;

pub fn nom_error_to_anyhow(err: NomErr<nom::error::Error<&str>>) -> anyhow::Error {
    match err {
        NomErr::Incomplete(needed) => anyhow::Error::msg(format!("Incomplete: {:?}", needed)),
        NomErr::Error(e) | NomErr::Failure(e) => anyhow::Error::msg(format!("Error: {}", e)),
    }
}
#[derive(Debug, PartialEq, Default)]
pub enum KickStyle {
    #[default]
    KickNow,
    KickBanNow,
    BanInit,
    TooManyIpInit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum DLobbyType {
    #[default]
    Solo,
    SoloIncognito,
    Duo,
    Multi,
}
impl std::str::FromStr for DLobbyType {
    type Err = MyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(DLobbyType::Solo),
            "2" => Ok(DLobbyType::Duo),
            "x" => Ok(DLobbyType::Multi),
            "1h" => Ok(DLobbyType::SoloIncognito),
            _ => Err(MyParseError),
        }
    }
}
impl Display for DLobbyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            DLobbyType::Solo => "1",
            DLobbyType::SoloIncognito => "1h",
            DLobbyType::Duo => "2",
            DLobbyType::Multi => "x",
        };
        write!(f, "{}", value)
    }
}

impl std::str::FromStr for KickStyle {
    type Err = MyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(KickStyle::KickNow),
            "2" => Ok(KickStyle::KickBanNow),
            "3" => Ok(KickStyle::BanInit),
            "4" => Ok(KickStyle::TooManyIpInit),
            _ => Err(MyParseError),
        }
    }
}
impl Display for KickStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            KickStyle::KickNow => 1,
            KickStyle::KickBanNow => 2,
            KickStyle::BanInit => 3,
            KickStyle::TooManyIpInit => 4,
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, PartialEq)]
pub struct MyParseError;

macro_rules! impl_from_str_enum {
    ($enum_name:ident { $($variant:ident),* }) => {

        #[derive(Debug, PartialEq, Copy, Clone, Default)]
        pub enum $enum_name{
            #[default]
            $(
                 $variant,
            )*
        }


        impl Display for $enum_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                $(
                    $enum_name::$variant => write!(f, lower!(stringify!($variant))),
                )*

                }
            }
        }

        impl std::str::FromStr for $enum_name {
            type Err = MyParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        lower!(stringify!($variant)) => Ok($enum_name::$variant),
                    )*
                    _ => Err(MyParseError),
                }
            }
        }
    };
}

impl_from_str_enum!(DLoginType { Nr, Reg, Ttm });

impl_from_str_enum!(DChallengeFail {
    Refuse,
    NoChall,
    CByOther,
    NoUser,
    COther
});

impl_from_str_enum!(DLoginStatus {
    NickInUse,
    Rlf,
    InvalidNick,
    ForbiddenNick
});

impl_from_str_enum!(DErrorType {
    VerNotOk,
    ServerFull
});
#[derive(Debug, Clone, Copy)]
pub struct PacketNumber(pub u32);

impl fmt::Display for PacketNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Parse for PacketNumber {
    fn parse(input: &str) -> IResult<&str, Self>
    where
        Self: Sized,
    {
        let (input, packet_number) = map_res(take_while(|c: char| c != ' '), u32::parse)(input)?;
        Ok((input, PacketNumber { 0: packet_number.1 }))
    }

    fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Parse for bool {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, key) = alt((char('f'), char('t')))(input)?;
        Ok((input, key == 't'))
    }

    fn as_string(&self) -> String {
        match self {
            true => "t".to_string(),
            false => "f".to_string(),
        }
    }
}

pub trait Parse {
    fn parse(input: &str) -> IResult<&str, Self>
    where
        Self: Sized;

    fn as_string(&self) -> String;
}

pub trait Packet {
    fn packet_number(&self) -> Option<PacketNumber>;
}

macro_rules! impl_parse {
    ($type:ty) => {
        impl Parse for $type {
            fn parse(input: &str) -> IResult<&str, Self> {
                map_res(take_while(|c: char| c != '\t' && c != '\n'), str::parse)(input)
            }

            fn as_string(&self) -> String {
                self.to_string()
            }
        }
    };
}

impl_parse!(usize);
impl_parse!(u8);
impl_parse!(u32);
impl_parse!(i32);
impl_parse!(i64);
impl_parse!(String);
impl_parse!(DLoginStatus);
impl_parse!(DChallengeFail);
impl_parse!(DErrorType);
impl_parse!(DLoginType);
impl_parse!(DLobbyType);
impl_parse!(KickStyle);

macro_rules! impl_parse_from_enum {
    ($enum_name:ident { $($value:expr => $variant:ident),* }) => {
        #[derive(Debug,PartialEq, Copy, Clone,Default)]
        pub enum $enum_name{
            #[default]
            $($variant,)*
        }

        impl Parse for $enum_name {
            fn parse(input: &str) -> IResult<&str, Self> {
                let (input,s) = i32::parse(input)?;
                match s {
                    $($value => Ok((input,$enum_name::$variant)),)*
                    _ => Err(nom::Err::Error(make_error(input, ErrorKind::Fail))),
                }
            }

            fn as_string(&self) -> String {
                match self{
                    $($enum_name::$variant => stringify!($value).to_string(),)*
                }
            }

        }
    };
}

impl_parse_from_enum!(WaterEvent {
    0 => BackToStart,
    1 => StayOnShore
});

impl_parse_from_enum!(Difficulty {
    1 => Easy,
    2 => Medium,
    3 => Hard
});

impl_parse_from_enum!(WeightEnd {
    0 => None,
    1 => Little,
    2 => Plenty
});

impl_parse_from_enum!(Scoring {
    0 => Score,
    1 => Track
});

impl_parse_from_enum!(Collision {
    0 => No,
    1 => Yes
});

impl_parse_from_enum!(TrackType {
    0 => All,
    1 => Basic,
    2 => Traditional,
    3 => Modern,
    4 => HoleInOne,
    5 => Short,
    6 => Long
});
/*
impl_parse_from_enum!(JoinLeaveReason {
    1 => StartedSP,
    2 => CreatedMP,
    3 => JoinedMP,
    4 => LeftLobby,
    5 => LostConnection
});*/

#[derive(Debug, PartialEq)]
pub struct SomeAsTab<T>(pub Option<T>);
#[derive(Debug, PartialEq)]
pub struct NoneAsTab<T>(pub Option<T>); //None == \t
#[derive(Debug, PartialEq)]
pub struct NonEmptyOption<T>(pub Option<T>); //None == -

impl<T: Parse> Parse for NonEmptyOption<T> {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(char('-'), |_| NonEmptyOption(None)),
            map(T::parse, |f| NonEmptyOption(Some(f))),
        ))(input)
    }

    fn as_string(&self) -> String {
        match &self.0 {
            Some(i) => i.as_string(),
            None => "-".to_string(),
        }
    }
}

impl<T: Parse> Parse for NoneAsTab<T> {
    fn parse(input: &str) -> IResult<&str, Self> {
        //Parser parses \t if we land on next arg its none
        if input.starts_with("\t") || input.starts_with("\n") {
            return Ok((input, NoneAsTab(None)));
        }

        alt((
            map(tag("\t"), |_| NoneAsTab(None)),
            //map(tag("\n"), |_| NoneAsTab(None)),
            map(T::parse, |f| NoneAsTab(Some(f))),
        ))(input)
    }

    fn as_string(&self) -> String {
        match &self.0 {
            Some(i) => format!("{}", i.as_string()),
            None => String::new(),
        }
    }
}
impl<T: Parse> Parse for SomeAsTab<T> {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(char('\t'), |_| SomeAsTab(None)),
            map(char('\n'), |_| SomeAsTab(None)),
            map(T::parse, |f| SomeAsTab(Some(f))),
        ))(input)
    }

    fn as_string(&self) -> String {
        match &self.0 {
            Some(i) => format!("\t{}", i.as_string()),
            None => String::new(),
        }
    }
}

impl Parse for char {
    fn parse(input: &str) -> IResult<&str, Self> {
        anychar(input)
    }

    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl<T: Parse> Parse for Vec<T> {
    fn parse(input: &str) -> IResult<&str, Vec<T>> {
        //        many1(<T>::parse)(input)
        separated_list1(char('\t'), T::parse)(input)
    }
    fn as_string(&self) -> String {
        self.iter()
            .map(|num| num.as_string())
            .collect::<Vec<_>>()
            .join("\t")

        //TODO remove last \t
    }
}

impl<T: Parse> Parse for Option<T> {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = opt(char('\t'))(input)?;
        alt((map(char('\n'), |_| None), map(T::parse, Some)))(input)
    }

    fn as_string(&self) -> String {
        match self {
            Some(i) => format!("{}", i.as_string()),
            None => String::new(),
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub id_username: String,
    pub value_1: String, //unkown
    pub rank: i32,
    pub lang: String,
    pub value_2: NonEmptyOption<String>,
    pub value_3: NonEmptyOption<String>,
}

impl Parse for User {
    fn parse(input: &str) -> IResult<&str, Self>
    where
        Self: Sized,
    {
        let mut parse = map_res(
            take_while(|c: char| c != '\t' && c != '^' && c != '\n'),
            str::parse,
        );
        let mut parse1 = map_res(
            take_while(|c: char| c != '\t' && c != '^' && c != '\n'),
            i32::parse,
        );
        let mut parse2 = map_res(
            take_while(|c: char| c != '\t' && c != '^' && c != '\n'),
            NonEmptyOption::parse,
        );
        let (input, id_username) = parse(input)?;
        let (input, _) = char('^')(input)?;
        let (input, value_1) = parse(input)?;
        let (input, _) = char('^')(input)?;
        let (input, rank) = parse1(input)?;
        let (input, _) = char('^')(input)?;
        let (input, lang) = parse(input)?;
        let (input, _) = char('^')(input)?;
        let (input, value_2) = parse2(input)?;
        let (input, _) = char('^')(input)?;
        let (input, value_3) = parse2(input)?;
        Ok((
            input,
            Self {
                id_username,
                value_1,
                rank: rank.1,
                lang,
                value_2: value_2.1,
                value_3: value_3.1,
            },
        ))
    }

    fn as_string(&self) -> String {
        format!(
            "{}^{}^{}^{}^{}^{}",
            self.id_username,
            self.value_1,
            self.rank,
            self.lang,
            self.value_2.as_string(),
            self.value_3.as_string(),
        )
    }
}

#[derive(Debug)]
pub enum JoinLeaveReason {
    StartedSP,
    CreatedMP(String),
    JoinedMP(String),
    LeftLobby,
    LostConnection,
}

#[derive(Debug)]
pub struct PlayerInfo(Vec<bool>);

impl Parse for PlayerInfo {
    fn parse(input: &str) -> IResult<&str, Self>
    where
        Self: Sized,
    {
        let mut ret = Vec::new();
        let (input, s) = String::parse(input)?;
        for i in s.chars() {
            match i {
                'f' => ret.push(true),
                't' => ret.push(false),
                _ => return Err(nom::Err::Error(make_error(input, ErrorKind::Fail))),
            }
        }
        Ok((input, PlayerInfo(ret)))
    }

    fn as_string(&self) -> String {
        let mut ret = String::new();
        for i in &self.0 {
            if *i == true {
                ret.push('t');
            } else {
                ret.push('f');
            }
        }
        ret
    }
}

impl Parse for JoinLeaveReason {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, s) = i32::parse(input)?;
        match s {
            1 => Ok((input, JoinLeaveReason::StartedSP)),
            2 => {
                let (input, _) = char('\t')(input)?;
                let (input, s) = String::parse(input)?;
                Ok((input, JoinLeaveReason::CreatedMP(s)))
            }
            3 => {
                let (input, _) = char('\t')(input)?;
                let (input, s) = String::parse(input)?;

                Ok((input, JoinLeaveReason::JoinedMP(s)))
            }
            4 => Ok((input, JoinLeaveReason::LeftLobby)),
            5 => Ok((input, JoinLeaveReason::LostConnection)),
            _ => Err(nom::Err::Error(make_error(input, ErrorKind::Fail))),
        }
    }
    fn as_string(&self) -> String {
        match self {
            JoinLeaveReason::StartedSP => "1".to_string(),
            JoinLeaveReason::CreatedMP(i) => format!("2\t{}", i),
            JoinLeaveReason::JoinedMP(i) => format!("3\t{}", i),
            JoinLeaveReason::LeftLobby => "4".to_string(),
            JoinLeaveReason::LostConnection => "5".to_string(),
        }
    }
}

mod tests {
    use crate::common::{Parse, User};

    #[test]
    fn user_parse() {
        let input = "3:~anonym-2893^wn^-1^de_DE^-^-";
        assert_eq!(
            User::parse(input).unwrap().1.as_string(),
            format!("\t{}", input)
        );
    }

    #[test]
    fn parse_test() {
        assert_eq!(32, <i32>::parse("32\t").unwrap().1)
    }
}

use rand::Rng;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

static CIPHER_CMDS: [&str; 68] = [
    "status\t",
    "basicinfo\t",
    "numberofusers\t",
    "users\t",
    "ownjoin\t",
    "joinfromgame\t",
    "say\t",
    "logintype\t",
    "login",
    "lobbyselect\t",
    "select\t",
    "back",
    "challenge\t",
    "cancel\t",
    "accept\t",
    "cfail\t",
    "nouser",
    "nochall",
    "cother",
    "cbyother",
    "refuse",
    "afail",
    "gsn\t",
    "lobby\tnc\t",
    "lobby\t",
    "lobby",
    "tracksetlist\t",
    "tracksetlist",
    "gamelist\t",
    "full\t",
    "add\t",
    "change\t",
    "remove\t",
    "gameinfo\t",
    "players",
    "owninfo\t",
    "game\tstarttrack\t",
    "game\tstartturn\t",
    "game\tstart",
    "game\tbeginstroke\t",
    "game\tendstroke\t",
    "game\tresetvoteskip",
    "game\t",
    "game",
    "quit",
    "join\t",
    "part\t",
    "cspt\t",
    "qmpt",
    "cspc\t",
    "jmpt\t",
    "tracklist\t",
    "Tiikoni",
    "Leonardo",
    "Ennaji",
    "Hoeg",
    "Darwin",
    "Dante",
    "ConTrick",
    "Dewlor",
    "Scope",
    "SuperGenuis",
    "Zwan",
    "\tT !\t",
    "\tcr\t",
    "rnop",
    "nop\t",
    "error",
];
const CIPHER_MAGIC_DEFAULT: i32 = 4;

pub struct GameCipher {
    cmds: Vec<String>,
}

impl GameCipher {
    pub fn new() -> Self {
        let mut cmds: Vec<String> = CIPHER_CMDS.iter().map(|&s| s.to_string()).collect();
        cmds.sort_by(|a, b| b.len().cmp(&a.len()));

        Self { cmds }
    }

    pub fn encrypt(&self, input: &str) -> String {
        if let Some(unused_char) = GameCipher::find_unused_char(input) {
            let cmds_length = self.cmds.len();
            let mut encrypted_input = input.to_string();

            for i in 0..cmds_length {
                while let Some(index) = encrypted_input.find(&self.cmds[i]) {
                    if !GameCipher::contains_char(&encrypted_input, index, unused_char) {
                        let replacement = format!("{}{}", unused_char, (b' ' + i as u8) as char);
                        encrypted_input
                            .replace_range(index..index + self.cmds[i].len(), &replacement);
                    }
                }
            }

            encrypted_input.insert_str(0, &unused_char.to_string());
            return encrypted_input;
        }

        input.to_string()
    }

    pub fn decrypt(&self, input: &str) -> String {
        if let Some(first_char) = input.chars().next() {
            let mut input = input.get(1..).unwrap_or_default().to_string();

            while let Some(char_index) = input.find(first_char) {
                if let Some(cmd_index) = input.get(char_index + 1..).and_then(|s| s.chars().next())
                {
                    if let Some(replacement) = self.cmds.get((cmd_index as usize).wrapping_sub(32))
                    {
                        input.replace_range(char_index..=char_index + 1, &replacement.to_string());
                    }
                }

                // Move past the processed substring
                input = input[char_index + 0..].to_string();
            }

            return input;
        }

        input.to_string()
    }

    fn contains_char(input: &str, pos: usize, c: char) -> bool {
        pos > 0 && input.chars().nth(pos - 1) == Some(c)
    }
    fn find_unused_char(input: &str) -> Option<char> {
        for c in 1..32 as u8 {
            let char_as_string = c as char;

            if !input.contains(char_as_string) {
                return Some(char_as_string);
            }
        }

        None
    }
}

pub struct ConnRandom {
    multiplier: u64,
    append: u64,
    mask: u64,
    nextseed: u64,
}

impl ConnRandom {
    pub fn new(seed: u64) -> ConnRandom {
        let multiplier = 0x5DEECE66D;
        let append = 0xB;
        let mask = (1u64 << 48) - 1;
        let nextseed = (seed ^ multiplier) & mask;

        ConnRandom {
            multiplier,
            append,
            mask,
            nextseed,
        }
    }

    pub fn next_int_min_max(&mut self, min: i32, max: i32) -> i32 {
        min + (self.next_int() % (max - min + 1) as i32)
    }

    fn next_int(&mut self) -> i32 {
        let next = self.next();
        if next < 0 {
            let next = -next;
            if next < 0 {
                return 0;
            }
            return next;
        }
        next
    }

    fn next(&mut self) -> i32 {
        self.nextseed = self
            .nextseed
            .wrapping_mul(self.multiplier)
            .wrapping_add(self.append)
            & self.mask;
        (self.nextseed >> 16) as i32
    }
}

pub struct Ciphers {
    pub game_cipher: Option<GameCipher>,
    pub conn_cipher: Option<ConnCipher>,
}

impl Ciphers {
    pub fn new(game_cipher: Option<GameCipher>, conn_cipher: Option<ConnCipher>) -> Self {
        Self {
            game_cipher,
            conn_cipher,
        }
    }
    pub fn none() -> Self {
        Self {
            game_cipher: None,
            conn_cipher: None,
        }
    }

    pub fn set_conn_cipher(&mut self, conn_cipher: Option<ConnCipher>) {
        self.conn_cipher = conn_cipher;
    }

    pub fn set_game_cipher(&mut self, game_cipher: Option<GameCipher>) {
        self.game_cipher = game_cipher;
    }
}

pub struct ConnCipher {
    magic: i32,
    seed: i32,
    randoms_ascii: [[i32; 125]; 2],
    randoms_other: [[i32; 1920]; 2],
}

impl ConnCipher {
    pub fn get_random_seed() -> i32 {
        rand::thread_rng().gen_range(100000000..=999999999)
    }

    pub fn new(magic: i32, seed: i32) -> ConnCipher {
        let mut ret = ConnCipher {
            magic,
            seed,
            randoms_ascii: [[-1; 125]; 2],
            randoms_other: [[-1; 1920]; 2],
        };

        let mut random = ConnRandom::new(seed as u64);

        let mut index = 1;
        while index <= 125 {
            let mut rand = random.next_int_min_max(1, 125) as usize;
            while ret.randoms_ascii[1][rand - 1] >= 0 {
                rand = random.next_int_min_max(1, 125) as usize;
            }
            ret.randoms_ascii[0][index - 1] = rand as i32;
            ret.randoms_ascii[1][rand - 1] = index as i32;
            index += 1;
        }

        index = 128;
        while index <= 2047 {
            let mut rand = random.next_int_min_max(128, 2047) as usize;
            while ret.randoms_other[1][rand - 128] >= 0 {
                rand = random.next_int_min_max(128, 2047) as usize;
            }
            ret.randoms_other[0][index - 128] = rand as i32;
            ret.randoms_other[1][rand - 128] = index as i32;
            index += 1;
        }
        ret
    }
    fn decrement<T>(val: T) -> T
    where
        T: Copy + PartialOrd + Sub<Output = T> + Neg<Output = T> + From<u8>,
    {
        let mut result = val;
        if result > T::from(13) {
            result = result - T::from(1);
        }

        if result > T::from(10) {
            result = result - T::from(1);
        }

        result
    }

    fn increment<T>(val: T) -> T
    where
        T: Copy + PartialOrd + Add<Output = T> + From<u8>,
    {
        let mut result = val;
        if result >= T::from(10) {
            result = result + T::from(1);
        }

        if result >= T::from(13) {
            result = result + T::from(1);
        }

        result
    }

    fn magic_mod2<T>(val1: T, val2: T, min: T, max: T) -> T
    where
        T: Copy
            + PartialOrd
            + Add<Output = T>
            + Mul<Output = T>
            + Sub<Output = T>
            + Neg<Output = T>
            + Div<Output = T>
            + Rem<Output = T>
            + Mul<Output = T>
            + From<u8>,
    {
        Self::magic_mod(val1 + val2, min, max)
    }

    fn magic_mod<T>(val: T, min: T, max: T) -> T
    where
        T: Copy
            + PartialOrd
            + Add<Output = T>
            + Mul<Output = T>
            + Sub<Output = T>
            + Neg<Output = T>
            + Div<Output = T>
            + Rem<Output = T>
            + Mul<Output = T>
            + From<u8>,
    {
        let mut val = val;
        let mut max = max;
        let min = min;

        max = max - min;
        val = val - min;
        let modulus = max + T::from(1);

        if val > max {
            val = val % modulus;
        } else if val < T::from(0) {
            let var5 = (-val - T::from(1)) / modulus + T::from(1);
            val = val + var5 * modulus;
        }

        val = val + min;
        val
    }

    pub fn encrypt(&self, input: &str) -> String {
        let first_random = rand::thread_rng().gen_range(1..=125);
        let last_random = rand::thread_rng().gen_range(1..=125);
        self.encrypt_non_random(first_random, last_random, input)
    }
    fn encrypt_non_random(&self, first_random: i32, last_random: i32, input: &str) -> String {
        let input_chars: Vec<char> = input.chars().collect();
        let input_length = input_chars.len();
        let mut output = String::with_capacity(input_length + 2);

        let rand_mod = Self::magic_mod(first_random, 1, input_length as i32 + 1);
        output.push(Self::increment(first_random as u8) as char);

        let mut seedling = self.seed % 99 - 49 + first_random - last_random;
        for (index, &cur_char) in input_chars.iter().enumerate() {
            let mut cur_char = cur_char as i32;

            if rand_mod == index as i32 + 1 {
                output.push(Self::increment(last_random as u8) as char);
            }

            if cur_char >= 1 && cur_char <= 127 {
                if cur_char as u8 != b'\n' && cur_char as u8 != b'\r' {
                    cur_char = Self::decrement(cur_char);
                    cur_char = Self::magic_mod2(cur_char, seedling, 1, 125);
                    seedling += 1;
                    cur_char = self.randoms_ascii[0][(cur_char - 1) as usize];
                    cur_char = Self::increment(cur_char);
                    if cur_char >= 14 && cur_char <= 127 {
                        cur_char = Self::magic_mod2(cur_char, self.magic - 9, 14, 127);
                    }
                }
            } else if cur_char >= 128 && cur_char <= 2047 {
                cur_char = Self::magic_mod2(cur_char, seedling, 128, 2047);
                seedling += 2;
                cur_char = self.randoms_other[0][(cur_char - 128) as usize];
            }

            output.push(Self::increment(cur_char as u8 - 2) as char); //TODO figure why this is
                                                                      //off by 2
            seedling += 1;
        }
        if rand_mod == input_length as i32 + 1 {
            output.push(Self::increment(last_random as u8) as char);
        }
        output
    }

    pub fn decrypt(&self, input: &str) -> String {
        let input_chars: Vec<char> = input.chars().collect();
        let input_length = input_chars.len();
        let mut output = String::with_capacity(input_length - 2);

        let first_random = Self::decrement(input_chars[0] as i32);
        let rand_mod = Self::magic_mod(first_random, 1, input_length as i32 - 1);
        let last_random = Self::decrement(input_chars[rand_mod as usize] as i32);

        let mut seedling = last_random - first_random - (self.seed % 99 - 49);
        let orig_input_length = if rand_mod < input_length as i32 - 1 {
            input_length
        } else {
            input_length - 1
        };

        for index in 1..orig_input_length {
            if index == rand_mod as usize {
                continue; // Skip this iteration
            }

            let mut cur_char = input_chars[index] as i32;

            if cur_char >= 1 && cur_char <= 127 {
                if cur_char as u8 != b'\n' && cur_char as u8 != b'\r' {
                    if cur_char >= 14 && cur_char <= 127 {
                        cur_char = Self::magic_mod2(cur_char, 9 - self.magic, 14, 127);
                    }

                    cur_char = Self::decrement(cur_char);
                    cur_char = self.randoms_ascii[1][(cur_char - 1) as usize];
                    cur_char = Self::magic_mod2(cur_char, seedling, 1, 125);
                    seedling -= 1;
                    cur_char = Self::increment(cur_char);
                }
            } else if cur_char >= 128 && cur_char <= 2047 {
                cur_char = self.randoms_other[1][(cur_char - 128) as usize];
                cur_char = Self::magic_mod2(cur_char, seedling, 128, 2047);
                seedling -= 2;
            }

            output.push(Self::increment(cur_char as u8 - 2) as char); //TODO
            seedling -= 1;
        }

        output
    }
}

mod tests {

    use super::ConnCipher;
    use super::GameCipher;
    #[test]
    fn encrypt_conn_test() {
        let conn = ConnCipher::new(4, 148153586);
        let txt = conn.encrypt("c new\n");
        let lol = conn.decrypt(&txt);

        assert_eq!("c new\n", conn.decrypt(&txt));
    }

    #[test]
    fn encrypt_game_test() {
        //TODO
        let cipher = GameCipher::new();
        assert_eq!(
            cipher.encrypt("game\tbeginstroke\t70q4\n"),
            "\u{1}\u{1}!70q4\n"
        );
        assert_eq!(
            cipher.decrypt(&cipher.encrypt("game\tbeginstroke\t7ors\n")),
            "game\tbeginstroke\t7ors\n"
        );
    }
}

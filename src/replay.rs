use serde::Serialize;
use std::{
    array::TryFromSliceError,
    fmt::{self},
    time::Duration,
};

#[derive(Debug)]
pub enum Mode {
    Marathon,
    Master,
    Normal,
    Konoha(KonohaDifficulty),
    Shiranui(u8, u8),
    Asuka,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Rule {
    Standard = 0,
    Tgm = 1,
}

impl From<u8> for Rule {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Rule::Standard,
            _ => Rule::Tgm,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum Mod {
    MaxG,
    Daily,
    Easy,
    Big,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum KonohaDifficulty {
    Easy,
    Hard,
}

impl From<u8> for KonohaDifficulty {
    fn from(value: u8) -> Self {
        match value {
            0x00 => KonohaDifficulty::Easy,
            _ => KonohaDifficulty::Hard,
        }
    }
}

#[derive(Debug)]
pub struct Replay {
    pub mode: Mode,
    pub rule: Rule,
    pub steamid: u64,
    pub timestamp: u64,
    pub modifiers: Vec<Mod>,
    pub score: u32,
    pub seed: u32,
    pub time: Duration,
    pub level: u32,
    pub bravo: u8,
    pub opponent: Option<Opponent>,
    // skin
}

#[derive(Debug)]
pub struct Opponent {
    seed: u32,
    rule: Rule,
    // skin
    // bravo?
}

#[derive(Debug, Clone)]
pub enum ReplayError {
    Slice(TryFromSliceError),
    Parse,
}

//TODO improve
impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReplayError::Slice(e) => e.fmt(f),
            ReplayError::Parse => write!(f, "Error parsing the modifier"),
        }
    }
}

fn parse_modifier(byte: u8) -> Vec<Mod> {
    let mut r = Vec::with_capacity(6);
    if byte & 0b01000000 == 0b01000000 {
        println!("Daily???");
        r.push(Mod::Daily);
    }

    if byte & 0b00110000 == 0b00110000 {
        r.push(Mod::Easy);
    }

    if byte & 0b00001100 == 0b00001100 {
        println!("This shouldn't be used this is epic !!!");
    }

    if byte & 0b00000010 == 0b00000010 {
        r.push(Mod::Big);
    }

    if byte & 0b00000001 == 0b00000001 {
        r.push(Mod::MaxG);
    }

    r
}

fn parse_mode(
    alt_byte: u8,
    mode_byte: u8,
    shiranui_points_byte: u8,
    shiranui_tier_byte: u8,
    rule: &Rule,
) -> Option<Mode> {
    match mode_byte {
        0x0 if *rule == Rule::Standard => Some(Mode::Marathon),
        0x00 if *rule == Rule::Tgm => Some(Mode::Normal),
        0x01 => Some(Mode::Master),
        0x03 => Some(Mode::Konoha(KonohaDifficulty::from(alt_byte))),
        0x04 => Some(Mode::Shiranui(shiranui_tier_byte, shiranui_points_byte)),
        0x05 => Some(Mode::Asuka),
        _ => None,
    }
}

//TODO test garbage type in replay?
impl Replay {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ReplayError> {
        let steam_id_bytes = &bytes[0x10..0x18].try_into().map_err(ReplayError::Slice)?;

        let timestamp_bytes = &bytes[0x18..0x20].try_into().map_err(ReplayError::Slice)?;
        let alt_byte = bytes[0x20];
        let is_versus = alt_byte == 0x03;

        let mode_byte = bytes[0x24];
        let shiranui_tier_byte = bytes[0x48];
        let shiranui_points_byte = bytes[0x0C];

        let rule_player_byte = bytes[0x28];
        let rule_player = Rule::from(rule_player_byte);

        let mode = parse_mode(
            alt_byte,
            mode_byte,
            shiranui_points_byte,
            shiranui_tier_byte,
            &rule_player,
        )
        .ok_or(ReplayError::Parse)?;

        let modifier_byte = bytes[0x30];
        let modifiers = parse_modifier(modifier_byte);

        let p1_seed = &bytes[0x34..0x38].try_into().map_err(ReplayError::Slice)?;

        let frame_time_bytes = &bytes[0x38..0x3C].try_into().map_err(ReplayError::Slice)?;
        let time = Duration::from_millis((100 / 6 * u32::from_le_bytes(*frame_time_bytes)).into());
        let level_bytes = &bytes[0x3C..0x40].try_into().map_err(ReplayError::Slice)?;
        let score_bytes = &bytes[0x40..0x44].try_into().map_err(ReplayError::Slice)?;

        let bravo = bytes[0x44];

        let opponent = if is_versus {
            let seed_bytes = &bytes[0x104..0x108].try_into().map_err(ReplayError::Slice)?;
            let rule_aux_byte = bytes[0x2C];
            Some(Opponent {
                seed: u32::from_le_bytes(*seed_bytes),
                rule: Rule::from(rule_aux_byte),
            })
        } else {
            None
        };

        Ok(Replay {
            mode,
            rule: rule_player,
            steamid: u64::from_le_bytes(*steam_id_bytes),
            timestamp: u64::from_le_bytes(*timestamp_bytes),
            modifiers,
            score: u32::from_le_bytes(*score_bytes),
            time,
            level: u32::from_le_bytes(*level_bytes),
            bravo,
            seed: u32::from_le_bytes(*p1_seed),
            opponent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitflags() {
        assert_eq!(parse_modifier(0b11111111), vec![
            Mod::Daily,
            Mod::Easy,
            Mod::Big,
            Mod::MaxG
        ]);
        assert_eq!(parse_modifier(0b10000000), vec![]);
        assert_eq!(parse_modifier(0b01000000), vec![Mod::Daily]);
        assert_eq!(parse_modifier(0b00110000), vec![Mod::Easy]);
        assert_eq!(parse_modifier(0b00110010), vec![Mod::Easy, Mod::Big]);
        assert_eq!(parse_modifier(0b00001100), vec![]);
        assert_eq!(parse_modifier(0b00000010), vec![Mod::Big]);
        assert_eq!(parse_modifier(0b00000001), vec![Mod::MaxG]);
    }
}

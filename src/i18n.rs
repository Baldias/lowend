//! UI strings. Add a language: add a `Lang` variant and the compiler lists every missing string.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Lang {
    It,
    En,
}

#[derive(Clone, Copy)]
pub enum Msg {
    PlayAString,
    NoMic,
    Standard,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::It, Lang::En];

    /// System language, falling back to English.
    pub fn detect() -> Self {
        match sys_locale::get_locale() {
            Some(l) if l.starts_with("it") => Lang::It,
            _ => Lang::En,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::It => "IT",
            Lang::En => "EN",
        }
    }

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&l| l == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn t(self, msg: Msg) -> &'static str {
        use {Lang::*, Msg::*};
        match (self, msg) {
            (It, PlayAString) => "Suona una corda",
            (En, PlayAString) => "Play a string",
            (It, NoMic) => "Microfono non disponibile",
            (En, NoMic) => "Microphone unavailable",
            (It, Standard) => "STANDARD",
            (En, Standard) => "STANDARD",
        }
    }
}

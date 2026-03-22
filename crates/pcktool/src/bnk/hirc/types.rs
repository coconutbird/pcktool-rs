//! HIRC enums and type definitions.

use core::fmt;

/// HIRC item type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HircType {
    State = 1,
    Sound = 2,
    Action = 3,
    Event = 4,
    RanSeqCntr = 5,
    SwitchCntr = 6,
    ActorMixer = 7,
    Bus = 8,
    LayerCntr = 9,
    MusicSegment = 10,
    MusicTrack = 11,
    MusicSwitch = 12,
    MusicRanSeq = 13,
    Attenuation = 14,
    DialogueEvent = 15,
    FeedbackBus = 16,
    FeedbackNode = 17,
    FxShareSet = 18,
    FxCustom = 19,
    AuxBus = 20,
    LfoModulator = 21,
    EnvelopeModulator = 22,
    AudioDevice = 23,
}

impl HircType {
    /// Try to convert a raw `u8` to a `HircType`.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::State),
            2 => Some(Self::Sound),
            3 => Some(Self::Action),
            4 => Some(Self::Event),
            5 => Some(Self::RanSeqCntr),
            6 => Some(Self::SwitchCntr),
            7 => Some(Self::ActorMixer),
            8 => Some(Self::Bus),
            9 => Some(Self::LayerCntr),
            10 => Some(Self::MusicSegment),
            11 => Some(Self::MusicTrack),
            12 => Some(Self::MusicSwitch),
            13 => Some(Self::MusicRanSeq),
            14 => Some(Self::Attenuation),
            15 => Some(Self::DialogueEvent),
            16 => Some(Self::FeedbackBus),
            17 => Some(Self::FeedbackNode),
            18 => Some(Self::FxShareSet),
            19 => Some(Self::FxCustom),
            20 => Some(Self::AuxBus),
            21 => Some(Self::LfoModulator),
            22 => Some(Self::EnvelopeModulator),
            23 => Some(Self::AudioDevice),
            _ => None,
        }
    }
}

impl fmt::Display for HircType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State => write!(f, "State"),
            Self::Sound => write!(f, "Sound"),
            Self::Action => write!(f, "Action"),
            Self::Event => write!(f, "Event"),
            Self::RanSeqCntr => write!(f, "RanSeqCntr"),
            Self::SwitchCntr => write!(f, "SwitchCntr"),
            Self::ActorMixer => write!(f, "ActorMixer"),
            Self::Bus => write!(f, "Bus"),
            Self::LayerCntr => write!(f, "LayerCntr"),
            Self::MusicSegment => write!(f, "MusicSegment"),
            Self::MusicTrack => write!(f, "MusicTrack"),
            Self::MusicSwitch => write!(f, "MusicSwitch"),
            Self::MusicRanSeq => write!(f, "MusicRanSeq"),
            Self::Attenuation => write!(f, "Attenuation"),
            Self::DialogueEvent => write!(f, "DialogueEvent"),
            Self::FeedbackBus => write!(f, "FeedbackBus"),
            Self::FeedbackNode => write!(f, "FeedbackNode"),
            Self::FxShareSet => write!(f, "FxShareSet"),
            Self::FxCustom => write!(f, "FxCustom"),
            Self::AuxBus => write!(f, "AuxBus"),
            Self::LfoModulator => write!(f, "LfoModulator"),
            Self::EnvelopeModulator => write!(f, "EnvelopeModulator"),
            Self::AudioDevice => write!(f, "AudioDevice"),
        }
    }
}

/// Action type (u16 in the binary format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionType(pub u16);

// Well-known action type constants
impl ActionType {
    pub const NONE: Self = Self(0x0000);
    pub const PLAY: Self = Self(0x0403);
    pub const PLAY_AND_CONTINUE: Self = Self(0x0503);
    pub const PLAY_EVENT: Self = Self(0x2103);
    pub const PLAY_EVENT_UNKNOWN_O: Self = Self(0x2303);
    pub const SET_STATE: Self = Self(0x1204);
    pub const SET_SWITCH: Self = Self(0x1901);
    pub const DUCK: Self = Self(0x1820);
    pub const NO_OP: Self = Self(0x4000);
    pub const STOP_EVENT: Self = Self(0x1511);
    pub const PAUSE_EVENT: Self = Self(0x1611);
    pub const RESUME_EVENT: Self = Self(0x1711);

    /// Determine the action category for dispatch.
    pub fn category(self) -> ActionCategory {
        // PlayEvent must be checked BEFORE Play
        if self == Self::PLAY_EVENT {
            return ActionCategory::PlayEvent;
        }
        if matches!(
            self,
            Self { 0: 0x0403 } | Self { 0: 0x0503 } | Self { 0: 0x2303 }
        ) {
            return ActionCategory::Play;
        }
        if self.is_active() {
            return ActionCategory::Active;
        }
        if self == Self::SET_STATE {
            return ActionCategory::State;
        }
        if self == Self::SET_SWITCH {
            return ActionCategory::Switch;
        }
        if self.is_game_param() {
            return ActionCategory::GameParam;
        }
        if self.is_value() {
            return ActionCategory::Value;
        }
        if self.is_bypass_fx() {
            return ActionCategory::BypassFX;
        }
        if self.is_seek() {
            return ActionCategory::Seek;
        }
        if self.is_none_params() {
            return ActionCategory::None;
        }
        ActionCategory::Unknown
    }

    fn is_stop(self) -> bool {
        matches!(self.0, 0x0102 | 0x0103 | 0x0104 | 0x0105 | 0x0108 | 0x0109)
    }
    fn is_pause(self) -> bool {
        matches!(self.0, 0x0202 | 0x0203 | 0x0204 | 0x0205 | 0x0208 | 0x0209)
    }
    fn is_resume(self) -> bool {
        matches!(self.0, 0x0302 | 0x0303 | 0x0304 | 0x0305 | 0x0308 | 0x0309)
    }
    fn is_active(self) -> bool {
        self.is_stop() || self.is_pause() || self.is_resume()
    }
    fn is_seek(self) -> bool {
        matches!(self.0, 0x1E02 | 0x1E03 | 0x1E04 | 0x1E05 | 0x1E08 | 0x1E09)
    }
    fn is_game_param(self) -> bool {
        matches!(self.0, 0x1302 | 0x1303 | 0x1402 | 0x1403)
    }
    fn is_value(self) -> bool {
        matches!(
            self.0,
            // Mute/Unmute
            0x0602 | 0x0603 | 0x0702 | 0x0703 | 0x0704 | 0x0705 | 0x0708 | 0x0709 |
            // SetVolume/ResetVolume
            0x0A02 | 0x0A03 | 0x0B02 | 0x0B03 | 0x0B04 | 0x0B05 | 0x0B08 | 0x0B09 |
            // SetPitch/ResetPitch
            0x0802 | 0x0803 | 0x0902 | 0x0903 | 0x0904 | 0x0905 | 0x0908 | 0x0909 |
            // SetLPF/ResetLPF
            0x0E02 | 0x0E03 | 0x0F02 | 0x0F03 | 0x0F04 | 0x0F05 | 0x0F08 | 0x0F09 |
            // SetHPF/ResetHPF
            0x2002 | 0x2003 | 0x3002 | 0x3003 | 0x3004 | 0x3005 | 0x3008 | 0x3009 |
            // SetBusVolume/ResetBusVolume
            0x0C02 | 0x0C03 | 0x0D02 | 0x0D03 | 0x0D04 | 0x0D08
        )
    }
    fn is_bypass_fx(self) -> bool {
        matches!(
            self.0,
            0x1A02
                | 0x1A03
                | 0x1B02
                | 0x1B03
                | 0x1B04
                | 0x1B05
                | 0x1B08
                | 0x1B09
                | 0x3302
                | 0x3303
                | 0x3402
                | 0x3403
                | 0x3404
                | 0x3405
                | 0x3502
                | 0x3503
                | 0x3602
                | 0x3603
                | 0x3604
                | 0x3605
                | 0x3702
                | 0x3703
                | 0x3704
                | 0x3705
        )
    }
    fn is_none_params(self) -> bool {
        matches!(
            self.0,
            0x1002 | 0x1102 | // UseState/UnuseState
            0x1C02 | 0x1C03 | // Break
            0x1D00 | 0x1D01 | 0x1D02 | 0x1D03 | 0x1B00 | 0x1B01 | // Trigger
            0x1820 | // Duck
            0x1F02 | 0x1F03 | // Release
            0x2202 | 0x2203 | // ResetPlaylist
            0x3102 | 0x3202 | 0x3204 | // SetFX
            0x1511 | 0x1611 | 0x1711 | // StopEvent/PauseEvent/ResumeEvent
            0x4000 | 0x0000 // NoOp/None
        )
    }
}

/// Action category — determines which params struct an action uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    None,
    PlayEvent,
    Play,
    Active,
    State,
    Switch,
    GameParam,
    Value,
    BypassFX,
    Seek,
    Event,
    Unknown,
}

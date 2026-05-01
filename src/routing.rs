use crate::midi_io::MidiPortRef;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TrackRouting {
    #[serde(default)]
    pub input_port: TrackPortSelection,
    #[serde(default)]
    pub output_port: TrackPortSelection,
    pub input_channel: MidiChannelFilter,
    pub output_channel: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TrackPortSelection {
    #[default]
    None,
    Default,
    Port(MidiPortRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TrackPortSelectionRepr {
    None,
    Default,
    Port { port: MidiPortRef },
}

impl TrackPortSelection {
    pub fn named(port: MidiPortRef) -> Self {
        Self::Port(port)
    }

    pub fn as_named_port(&self) -> Option<&MidiPortRef> {
        match self {
            Self::Port(port) => Some(port),
            Self::None | Self::Default => None,
        }
    }

    pub fn resolve<'a>(&'a self, default_port: Option<&'a MidiPortRef>) -> Option<&'a MidiPortRef> {
        match self {
            Self::None => None,
            Self::Default => default_port,
            Self::Port(port) => Some(port),
        }
    }

    pub fn cloned_resolved(&self, default_port: Option<&MidiPortRef>) -> Option<MidiPortRef> {
        self.resolve(default_port).cloned()
    }

    pub fn follows_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl Serialize for TrackPortSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr = match self {
            Self::None => TrackPortSelectionRepr::None,
            Self::Default => TrackPortSelectionRepr::Default,
            Self::Port(port) => TrackPortSelectionRepr::Port { port: port.clone() },
        };
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TrackPortSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TrackPortSelectionCompat {
            Tagged(TrackPortSelectionRepr),
            Legacy(Option<MidiPortRef>),
        }

        Ok(match TrackPortSelectionCompat::deserialize(deserializer)? {
            TrackPortSelectionCompat::Tagged(TrackPortSelectionRepr::None) => Self::None,
            TrackPortSelectionCompat::Tagged(TrackPortSelectionRepr::Default) => Self::Default,
            TrackPortSelectionCompat::Tagged(TrackPortSelectionRepr::Port { port }) => {
                Self::Port(port)
            }
            TrackPortSelectionCompat::Legacy(Some(port)) => Self::Port(port),
            TrackPortSelectionCompat::Legacy(None) => Self::None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MidiChannelFilter {
    #[default]
    Omni,
    Channel(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParticipantActorType {
    Human,
    Agent,
    Institution,
}

impl ParticipantActorType {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
            Self::Institution => "institution",
        }
    }

    pub fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "human" => Some(Self::Human),
            "agent" => Some(Self::Agent),
            "institution" => Some(Self::Institution),
            _ => None,
        }
    }
}

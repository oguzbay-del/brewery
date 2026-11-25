//! Container data types shared by the database and API crates.

use serde::{Deserialize, Serialize};

/// The lifecycle state stored in container.status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerStatus {
    /// Clean and ready for use.
    Clean,
    /// Filled and in circulation.
    Filled,
    /// Temporarily outside the production site.
    Out,
    /// Used and awaiting cleaning.
    Dirty,
    /// Under maintenance; cannot be filled.
    Maintenance,
    /// Permanently retired; cannot be filled.
    Retired,
}

impl ContainerStatus {
    /// All accepted database values, in stable API order.
    pub const ALL: &'static [Self] = &[
        Self::Clean,
        Self::Filled,
        Self::Out,
        Self::Dirty,
        Self::Maintenance,
        Self::Retired,
    ];

    /// Returns the database representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Filled => "filled",
            Self::Out => "out",
            Self::Dirty => "dirty",
            Self::Maintenance => "maintenance",
            Self::Retired => "retired",
        }
    }

    /// Parses a database value. Unknown values are not silently mapped to a state.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "clean" => Some(Self::Clean),
            "filled" => Some(Self::Filled),
            "out" => Some(Self::Out),
            "dirty" => Some(Self::Dirty),
            "maintenance" => Some(Self::Maintenance),
            "retired" => Some(Self::Retired),
            _ => None,
        }
    }

    /// Whether the database must reject a transition into filled from this state.
    #[must_use]
    pub const fn blocks_fill(self) -> bool {
        matches!(self, Self::Dirty | Self::Maintenance | Self::Retired)
    }
}

/// The two independent references that identify the latest container fill.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContainerFillReferences {
    /// Packaging run line that produced the fill.
    pub last_fill_line_id: Option<uuid::Uuid>,
    /// Traceable stock lot used for recall.
    pub last_lot_id: Option<uuid::Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_values_round_trip() {
        for status in ContainerStatus::ALL {
            assert_eq!(ContainerStatus::parse(status.as_str()), Some(*status));
        }
        assert!(ContainerStatus::parse("unknown").is_none());
    }

    #[test]
    fn unsafe_states_block_filling() {
        assert!(ContainerStatus::Dirty.blocks_fill());
        assert!(ContainerStatus::Maintenance.blocks_fill());
        assert!(ContainerStatus::Retired.blocks_fill());
        assert!(!ContainerStatus::Clean.blocks_fill());
        assert!(!ContainerStatus::Out.blocks_fill());
        assert!(!ContainerStatus::Filled.blocks_fill());
    }
}

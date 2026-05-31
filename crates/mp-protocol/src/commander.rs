//! Domain types: `Rank`, `Unit`, `Commander`.

use crate::error::{ProtocolError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Turkish military rank hierarchy (highest → lowest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rank {
    Maresal,
    Orgeneral,
    Korgeneral,
    Tumgeneral,
    Tuggeneral,
    Albay,
}

impl Rank {
    /// Parse a rank from a string. Accepts both Turkish ("Tümgeneral")
    /// and ASCII ("Tumgeneral") spellings since CSV input may be either.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "Maresal" => Ok(Self::Maresal),
            "Orgeneral" => Ok(Self::Orgeneral),
            "Korgeneral" => Ok(Self::Korgeneral),
            "Tümgeneral" | "Tumgeneral" => Ok(Self::Tumgeneral),
            "Tuggeneral" => Ok(Self::Tuggeneral),
            "Albay" => Ok(Self::Albay),
            other => Err(ProtocolError::InvalidRank(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Maresal => "Maresal",
            Self::Orgeneral => "Orgeneral",
            Self::Korgeneral => "Korgeneral",
            Self::Tumgeneral => "Tümgeneral",
            Self::Tuggeneral => "Tuggeneral",
            Self::Albay => "Albay",
        }
    }
}

/// A military unit — e.g. "107.Topçu Alayı, Siverek".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub id: Uuid,
    pub corps_number: i32,
    pub name: String,      // e.g. "107.Topçu"
    pub unit_type: String, // e.g. "Alayi" / "Tugayi" / "Tümeni" / "Kolordu"
    pub location: String,  // e.g. "Siverek"
    pub created_at: DateTime<Utc>,
}

impl Unit {
    pub fn new(corps_number: i32, name: String, unit_type: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            corps_number,
            name,
            unit_type,
            location,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commander {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub rank: Rank,
    pub public_key_pem: String,
    pub unit_id: Option<Uuid>,
    /// e.g. "https://commander-aylin:8443" — resolvable on the Docker network.
    pub network_address: String,
    pub created_at: DateTime<Utc>,
}

impl Commander {
    /// Create a new commander. The ID is generated automatically.
    pub fn new(
        full_name: String,
        email: String,
        rank: Rank,
        public_key_pem: String,
        network_address: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            full_name,
            email,
            rank,
            public_key_pem,
            unit_id: None,
            network_address,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_parse_roundtrip() {
        for r in [
            Rank::Maresal,
            Rank::Orgeneral,
            Rank::Korgeneral,
            Rank::Tumgeneral,
            Rank::Tuggeneral,
            Rank::Albay,
        ] {
            let s = r.as_str();
            let parsed = Rank::from_str(s).unwrap();
            assert_eq!(parsed, r);
        }
    }

    #[test]
    fn rank_handles_tumgeneral_with_dotted_u() {
        // Both Turkish "Tümgeneral" and ASCII "Tumgeneral" are accepted.
        assert_eq!(Rank::from_str("Tümgeneral").unwrap(), Rank::Tumgeneral);
        assert_eq!(Rank::from_str("Tumgeneral").unwrap(), Rank::Tumgeneral);
    }

    #[test]
    fn rank_rejects_invalid() {
        assert!(Rank::from_str("Pilot").is_err());
        assert!(Rank::from_str("").is_err());
    }

    #[test]
    fn commander_serialization() {
        let c = Commander::new(
            "Aylin Kaya".into(),
            "aylin@karakuvvetleri.mil.tr".into(),
            Rank::Tuggeneral,
            "MIIBIjAN...".into(),
            "https://commander-aylin:8443".into(),
        );
        let json = serde_json::to_string(&c).unwrap();
        let parsed: Commander = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.full_name, c.full_name);
        assert_eq!(parsed.rank, c.rank);
    }
}

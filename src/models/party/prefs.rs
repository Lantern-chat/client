use super::*;

use crate::models::Locale;

bitflags2! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PartyPrefsFlags: i32 {
        const DEFAULT = 0;
    }
}

impl_rkyv_for_bitflags!(pub PartyPrefsFlags: i32);
impl_serde_for_bitflags!(PartyPrefsFlags);
impl_sql_for_bitflags!(PartyPrefsFlags);
impl_schema_for_bitflags!(PartyPrefsFlags);

impl From<u64> for PartyPrefsFlags {
    fn from(value: u64) -> Self {
        PartyPrefsFlags::from_bits_truncate(value as _)
    }
}

impl Default for PartyPrefsFlags {
    fn default() -> Self {
        Self::DEFAULT
    }
}

mod preferences {
    decl_newtype_prefs! {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct PartyPreferences {
    /// Party locale (alias `locale`)
    #[serde(default, skip_serializing_if = "is_default", alias = "locale")]
    pub l: Locale,

    /// Party preferences flags (alias `flags`)
    #[serde(default, skip_serializing_if = "is_default", alias = "flags")]
    pub f: PartyPrefsFlags,
}

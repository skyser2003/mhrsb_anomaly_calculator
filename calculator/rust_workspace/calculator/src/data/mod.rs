use serde::{Deserialize, Serialize};

pub mod armor;
pub mod data_manager;
pub mod deco;
pub mod parser;
pub mod skill;

#[repr(u8)]
#[derive(Deserialize, Serialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "zh-Hans")]
    ChineseSimplified,
    #[serde(rename = "zh-Hant")]
    ChineseTraditional,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "ar")]
    Arabian,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "pt")]
    Portuguese,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KawaiiLabGroup {
    pub name: String,
    pub name_katakana: String,
    pub common_name: String,
    pub members: Vec<KawaiiLabMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KawaiiLabMember {
    pub name: String,
    pub name_kana: String,
    pub nickname: String,
    pub color: String,
    pub birthday: String,
    pub from: String,
    pub height: String,
    pub blood_type: Option<String>,
}

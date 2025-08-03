# データ構造リファクタリング提案

## 背景

現在のデータ構造は KAWAII LAB. 専用に設計されているが、=LOVE など他の事務所のアイドルグループを追加するために、より汎用的な構造に変更する必要がある。

## 現在の構造

```rust
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
```

## 提案する新構造

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdolGroup {
    pub agency: String,                 // 事務所名 (例: "KAWAII LAB.", "代々木アニメーション学院")
    pub name: String,                   // グループ名 (例: "FRUITS ZIPPER", "=LOVE")
    pub name_katakana: String,          // カタカナ名 (例: "フルーツジッパー", "イコールラブ")
    pub common_name: Option<String>,    // 通称 (例: "ふるっぱー", "イコラブ")
    pub debut_date: Option<String>,     // デビュー日 (例: "2018-08-22")
    pub website: Option<String>,        // 公式サイト
    pub members: Vec<IdolMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdolMember {
    pub name: String,                   // フルネーム (例: "月足 天音", "大谷映美里")
    pub name_kana: String,              // ひらがな読み (例: "つきあし あまね", "おおたに えみり")
    pub nickname: Option<String>,       // 愛称 (例: "あまねき") - グループによっては無い
    pub color: Option<String>,          // メンバーカラー (例: "赤") - グループによっては無い
    pub birthday: String,               // 誕生日 (例: "1999-10-26", "1998年3月15日")
    pub age: Option<u8>,               // 年齢 - 計算可能だが明示的に持つ場合
    pub from: String,                  // 出身地 (例: "福岡県柳川市", "東京都")
    pub height: String,                // 身長 (例: "153cm", "155cm")
    pub blood_type: Option<String>,    // 血液型 (例: "O", "O型")
    pub generation: Option<String>,     // 期生 (例: "1期生") - 坂道系などで使用
    pub status: Option<String>,        // ステータス (例: "active", "graduated") - 卒業メンバー管理用
}
```

## データ例

### KAWAII LAB. (既存データ)

```rust
IdolGroup {
    agency: "KAWAII LAB.".to_string(),
    name: "FRUITS ZIPPER".to_string(),
    name_katakana: "フルーツジッパー".to_string(),
    common_name: Some("ふるっぱー".to_string()),
    debut_date: None,
    website: None,
    members: vec![
        IdolMember {
            name: "月足 天音".to_string(),
            name_kana: "つきあし あまね".to_string(),
            nickname: Some("あまねき".to_string()),
            color: Some("赤".to_string()),
            birthday: "1999-10-26".to_string(),
            age: None,
            from: "福岡県柳川市".to_string(),
            height: "153cm".to_string(),
            blood_type: Some("O".to_string()),
            generation: None,
            status: Some("active".to_string()),
        },
        // ...
    ],
}
```

### =LOVE (新規追加データ)

```rust
IdolGroup {
    agency: "代々木アニメーション学院".to_string(),
    name: "=LOVE".to_string(),
    name_katakana: "イコールラブ".to_string(),
    common_name: Some("イコラブ".to_string()),
    debut_date: Some("2017-08-02".to_string()),
    website: None,
    members: vec![
        IdolMember {
            name: "大谷映美里".to_string(),
            name_kana: "おおたに えみり".to_string(),
            nickname: None,
            color: None,
            birthday: "1998-03-15".to_string(),
            age: Some(27),
            from: "東京都".to_string(),
            height: "155cm".to_string(),
            blood_type: Some("O".to_string()),
            generation: None,
            status: Some("active".to_string()),
        },
        // ...
    ],
}
```

## 移行計画

### Phase 1: 新構造体の追加
1. `src/types.rs` に新しい `IdolGroup` と `IdolMember` を追加
2. 既存の `KawaiiLabGroup` と `KawaiiLabMember` は残す（後方互換性）

### Phase 2: データ変換
1. `src/data.rs` で既存の KAWAII LAB. データを新構造に変換
2. =LOVE のデータを新構造で追加

### Phase 3: 検索機能の更新
1. `src/main.rs` の `members` ツールを新構造に対応
2. 事務所名でのフィルタリング機能を追加（オプション）

### Phase 4: クリーンアップ
1. 古い構造体を削除
2. テストの更新

## 検討事項

### 1. 誕生日フォーマット
- 統一フォーマット（ISO 8601: "YYYY-MM-DD"）推奨
- 既存データとの互換性のため、パース処理が必要

### 2. 年齢の扱い
- 誕生日から自動計算 vs 明示的な値
- データの鮮度管理が課題

### 3. 血液型フォーマット
- "O" vs "O型" の統一

### 4. 検索機能の拡張
```rust
// 事務所指定検索の例
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MembersRequest {
    pub query: String,
    pub agency: Option<String>,  // 事務所でフィルタ
}
```

## 利点

1. **拡張性**: 任意の事務所のグループを追加可能
2. **柔軟性**: オプショナルフィールドで異なるデータ形式に対応
3. **検索性**: 事務所横断的な検索が可能
4. **保守性**: 一貫したデータ構造

## 実装の優先度

1. **High**: 基本的な新構造体の追加
2. **High**: KAWAII LAB. データの移行
3. **High**: =LOVE データの追加
4. **Medium**: 検索機能の拡張（事務所フィルタ）
5. **Low**: 古い構造体の削除

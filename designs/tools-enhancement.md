# MCPツールの汎用性向上提案

## 現在のツール実装の分析

### 現在の `members` ツール

```rust
#[tool(description = "KAWAII LAB. の情報を取得する")]
async fn members(&self, params: Parameters<MembersRequest>) -> Result<CallToolResult, rmcp::ErrorData>
```

**問題点:**
1. **説明文が固定**: "KAWAII LAB. の情報を取得する" - 他の事務所に対応していない
2. **データソースが固定**: `get_kawaii_lab_data()` のみ
3. **検索対象が限定**: KAWAII LAB. のグループとメンバーのみ
4. **出力形式が単一**: JSON形式のみ

## 提案する改善案

### 1. 複数ツールアプローチ

事務所ごとに専用ツールを提供する方式：

```rust
#[tool(description = "KAWAII LAB. グループのメンバー情報を検索")]
async fn kawaii_lab_members(&self, params: Parameters<MembersRequest>) -> Result<CallToolResult, rmcp::ErrorData>

#[tool(description = "=LOVE グループのメンバー情報を検索")]
async fn equal_love_members(&self, params: Parameters<MembersRequest>) -> Result<CallToolResult, rmcp::ErrorData>

#[tool(description = "すべての事務所のアイドルグループを横断検索")]
async fn search_all_idols(&self, params: Parameters<GlobalSearchRequest>) -> Result<CallToolResult, rmcp::ErrorData>
```

**利点:**
- 事務所特化の検索が可能
- 説明文が明確
- 段階的実装が容易

**欠点:**
- ツール数が増加
- コード重複

### 2. 統一ツール + フィルターアプローチ（推奨）

単一ツールで事務所フィルタリングを提供：

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct IdolSearchRequest {
    /// 検索クエリ（グループ名、メンバー名、愛称など）
    pub query: String,
    /// 事務所フィルター（省略時は全事務所対象）
    pub agency: Option<String>,
    /// 検索対象の種別
    pub search_type: Option<SearchType>,
    /// 結果の表示形式
    pub format: Option<OutputFormat>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum SearchType {
    /// メンバーのみ検索
    Members,
    /// グループのみ検索
    Groups,
    /// 両方検索（デフォルト）
    All,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum OutputFormat {
    /// 詳細情報（デフォルト）
    Detailed,
    /// 簡潔な情報
    Summary,
    /// 名前のみ
    Names,
}
```

### 3. 拡張ツールセット

追加機能を提供するツール群：

```rust
#[tool(description = "アイドルグループのメンバー情報を検索")]
async fn search_idols(&self, params: Parameters<IdolSearchRequest>) -> Result<CallToolResult, rmcp::ErrorData>

#[tool(description = "登録されている全事務所の一覧を取得")]
async fn list_agencies(&self, params: Parameters<EmptyRequest>) -> Result<CallToolResult, rmcp::ErrorData>

#[tool(description = "指定事務所の全グループを取得")]
async fn list_groups(&self, params: Parameters<AgencyRequest>) -> Result<CallToolResult, rmcp::ErrorData>

#[tool(description = "メンバーの詳細情報を取得")]
async fn get_member_info(&self, params: Parameters<MemberDetailRequest>) -> Result<CallToolResult, rmcp::ErrorData>

#[tool(description = "グループの詳細情報を取得")]
async fn get_group_info(&self, params: Parameters<GroupDetailRequest>) -> Result<CallToolResult, rmcp::ErrorData>
```

## 実装例

### メインの検索ツール

```rust
#[tool(description = "アイドルグループとメンバーの情報を検索")]
async fn search_idols(
    &self,
    params: Parameters<IdolSearchRequest>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let request = &params.0;
    let query = &request.query;
    
    // 全データを取得（複数事務所対応）
    let all_groups = get_all_idol_data();
    
    // 事務所フィルタリング
    let filtered_groups: Vec<_> = if let Some(agency) = &request.agency {
        all_groups.iter().filter(|g| g.agency == *agency).collect()
    } else {
        all_groups.iter().collect()
    };
    
    let matcher = SkimMatcherV2::default();
    let mut matches = Vec::new();
    
    for group in filtered_groups {
        // 検索タイプに応じた処理
        match request.search_type.as_ref().unwrap_or(&SearchType::All) {
            SearchType::Groups => {
                if let Some(score) = search_group(&matcher, group, query) {
                    matches.push((SearchResult::Group(group.clone()), score));
                }
            }
            SearchType::Members => {
                for member in &group.members {
                    if let Some(score) = search_member(&matcher, member, query) {
                        matches.push((SearchResult::Member {
                            member: member.clone(),
                            group: group.name.clone(),
                            agency: group.agency.clone(),
                        }, score));
                    }
                }
            }
            SearchType::All => {
                // グループとメンバー両方を検索
                if let Some(score) = search_group(&matcher, group, query) {
                    matches.push((SearchResult::Group(group.clone()), score));
                }
                for member in &group.members {
                    if let Some(score) = search_member(&matcher, member, query) {
                        matches.push((SearchResult::Member {
                            member: member.clone(),
                            group: group.name.clone(),
                            agency: group.agency.clone(),
                        }, score));
                    }
                }
            }
        }
    }
    
    // スコア順でソート
    matches.sort_by(|a, b| b.1.cmp(&a.1));
    
    // 出力形式に応じたフォーマット
    let output = format_results(matches, request.format.as_ref().unwrap_or(&OutputFormat::Detailed))?;
    
    Ok(CallToolResult::success(vec![Content::text(output)]))
}
```

### 補助ツール

```rust
#[tool(description = "登録されている全事務所の一覧を取得")]
async fn list_agencies(&self, _params: Parameters<EmptyRequest>) -> Result<CallToolResult, rmcp::ErrorData> {
    let agencies: HashSet<String> = get_all_idol_data()
        .iter()
        .map(|g| g.agency.clone())
        .collect();
    
    let mut agency_list: Vec<String> = agencies.into_iter().collect();
    agency_list.sort();
    
    let output = agency_list.join("\n");
    Ok(CallToolResult::success(vec![Content::text(format!(
        "登録されている事務所:\n{}", output
    ))]))
}

#[tool(description = "指定事務所の全グループを取得")]
async fn list_groups(&self, params: Parameters<AgencyRequest>) -> Result<CallToolResult, rmcp::ErrorData> {
    let agency = &params.0.agency;
    let groups: Vec<String> = get_all_idol_data()
        .iter()
        .filter(|g| g.agency == *agency)
        .map(|g| format!("{} ({})", g.name, g.common_name.as_ref().unwrap_or(&"".to_string())))
        .collect();
    
    if groups.is_empty() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "事務所「{}」のグループは見つかりませんでした", agency
        ))]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{}のグループ:\n{}", agency, groups.join("\n")
        ))]))
    }
}
```

## データ取得の統一化

```rust
// data.rs に追加
pub fn get_all_idol_data() -> Vec<IdolGroup> {
    let mut all_groups = Vec::new();
    all_groups.extend(get_kawaii_lab_data());
    all_groups.extend(get_equal_love_data());
    // 今後他の事務所データも追加
    all_groups
}

pub fn get_equal_love_data() -> Vec<IdolGroup> {
    vec![
        IdolGroup {
            agency: "代々木アニメーション学院".to_string(),
            name: "=LOVE".to_string(),
            name_katakana: "イコールラブ".to_string(),
            common_name: Some("イコラブ".to_string()),
            debut_date: Some("2017-08-02".to_string()),
            website: None,
            members: vec![
                // =LOVEのメンバーデータ
            ],
        }
    ]
}
```

## 段階的実装計画

### Phase 1: 基本的な汎用化
1. `IdolSearchRequest` の実装
2. 既存 `members` ツールの `search_idols` への置き換え
3. 事務所フィルタリング機能の追加

### Phase 2: 補助ツールの追加
1. `list_agencies` ツールの実装
2. `list_groups` ツールの実装
3. 出力フォーマットオプションの実装

### Phase 3: 高度な機能
1. `get_member_info` と `get_group_info` の実装
2. 検索結果のランキング改善
3. 部分マッチ vs 完全マッチの制御

## 使用例

```
# 全事務所からCANDY TUNEを検索
search_idols({"query": "CANDY TUNE"})

# KAWAII LAB.のメンバーのみ検索
search_idols({"query": "あまねき", "agency": "KAWAII LAB.", "search_type": "Members"})

# 事務所一覧を取得
list_agencies({})

# KAWAII LAB.のグループ一覧
list_groups({"agency": "KAWAII LAB."})
```

## 利点

1. **段階的移行**: 既存機能を壊さずに拡張可能
2. **柔軟性**: 様々な検索ニーズに対応
3. **発見性**: `list_agencies` で利用可能な事務所がわかる
4. **保守性**: 統一されたデータ取得とフォーマット処理

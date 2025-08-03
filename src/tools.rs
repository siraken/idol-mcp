use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rmcp::types::tool::{Tool, ToolRequest};
use serde_json::{json, Value};

use crate::data::get_kawaii_lab_data;
use crate::types::KawaiiLabGroup;

pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "members".to_string(),
            description: Some("KAWAII LAB. の情報を取得する".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "グループ名 or グループ名の通称 or メンバー名 or メンバーの愛称"
                    }
                },
                "required": ["query"]
            }),
        }
    ]
}

pub async fn handle_tool_request(request: ToolRequest) -> Result<Value, String> {
    match request.name.as_str() {
        "members" => handle_members_request(request).await,
        _ => Err(format!("Unknown tool: {}", request.name)),
    }
}

async fn handle_members_request(request: ToolRequest) -> Result<Value, String> {
    let query = request.arguments
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "query parameter is required".to_string())?;

    let data = get_kawaii_lab_data();
    let matcher = SkimMatcherV2::default();
    
    let mut best_match: Option<(&KawaiiLabGroup, i64)> = None;
    
    for group in &data {
        let mut max_score = 0i64;
        
        if let Some(score) = matcher.fuzzy_match(&group.name, query) {
            max_score = max_score.max(score);
        }
        
        if let Some(score) = matcher.fuzzy_match(&group.common_name, query) {
            max_score = max_score.max(score);
        }
        
        for member in &group.members {
            if let Some(score) = matcher.fuzzy_match(&member.name, query) {
                max_score = max_score.max(score);
            }
            
            if let Some(score) = matcher.fuzzy_match(&member.nickname, query) {
                max_score = max_score.max(score);
            }
        }
        
        if max_score > 0 {
            if best_match.is_none() || best_match.as_ref().unwrap().1 < max_score {
                best_match = Some((group, max_score));
            }
        }
    }
    
    if let Some((group, _)) = best_match {
        let members_json = serde_json::to_value(&group.members)
            .map_err(|e| format!("Failed to serialize members: {}", e))?;
        
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("members of {}: {}", group.name, members_json)
            }]
        }))
    } else {
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("グループ {} は見つかりませんでした", query)
            }]
        }))
    }
}

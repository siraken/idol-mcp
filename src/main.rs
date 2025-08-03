mod data;
mod types;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{CallToolResult, Content, ErrorCode},
    tool, tool_handler, tool_router,
    transport::async_rw::AsyncRwTransport,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, stdout};

use crate::data::get_kawaii_lab_data;
use crate::types::KawaiiLabGroup;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MembersRequest {
    /// グループ名 or グループ名の通称 or メンバー名 or メンバーの愛称
    pub query: String,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for KawaiiLabServer {}

#[derive(Clone)]
pub struct KawaiiLabServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl KawaiiLabServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "KAWAII LAB. の情報を取得する")]
    async fn members(
        &self,
        params: Parameters<MembersRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let query = &params.0.query;
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
            let members_json =
                serde_json::to_string(&group.members).map_err(|e| rmcp::ErrorData {
                    message: format!("Failed to serialize members: {}", e).into(),
                    code: ErrorCode::INTERNAL_ERROR,
                    data: None,
                })?;

            Ok(CallToolResult::success(vec![Content::text(format!(
                "members of {}: {}",
                group.name, members_json
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "グループ {} は見つかりませんでした",
                query
            ))]))
        }
    }
}

#[tokio::main]
async fn main() {
    // トレーシングログを標準エラー出力に送る（標準出力はMCP通信用）
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting KAWAII LAB. MCP server...");

    let server = KawaiiLabServer::new();
    let transport = AsyncRwTransport::new_server(stdin(), stdout());

    tracing::info!("KAWAII LAB. MCP server is running on stdio");

    let _running = server.serve(transport).await.unwrap();

    // Server will run until the transport is closed
    tokio::signal::ctrl_c().await.unwrap();
    tracing::info!("Shutting down...");
}

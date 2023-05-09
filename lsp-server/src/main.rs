use std::{collections::HashMap, hash::Hash, string, sync::Mutex};

use dashmap::DashMap;
use tower_lsp::{
    jsonrpc,
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, InitializeParams, InitializeResult,
        InitializedParams, MessageType, ServerCapabilities, ServerInfo, Url,
        WorkDoneProgressOptions,
    },
    Client, LanguageServer, LspService, Server,
    {jsonrpc::Error, jsonrpc::Result},
};

use serde::{de, de::Error as Error_, Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
struct Backend {
    client: Client,
    import_map: DashMap<Url, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let server_info = Some(ServerInfo {
            name: "rust-fixer-lsp".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        });

        let capabilities = ServerCapabilities {
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
                resolve_provider: Some(true),
            })),
            ..Default::default()
        };

        Ok(InitializeResult {
            server_info,
            capabilities,
        })
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let CodeActionParams {
            range,
            context,
            text_document,
            ..
        } = params;

        // println!("{:?}", context);
        // text_document.uri

        let response = vec![CodeActionOrCommand::CodeAction(CodeAction {
            title: "fixxxxer".to_string(),
            is_preferred: Some(true),
            kind: Some(CodeActionKind::QUICKFIX),

            ..Default::default()
        })];

        Ok(Some(response))
    }

    async fn code_action_resolve(&self, mut param: CodeAction) -> Result<CodeAction> {
        param.is_preferred = Some(true);
        Result::Ok(param)
    }

    async fn initialized(&self, _: InitializedParams) {
        if let jsonrpc::Result::Ok(Some(workspace)) = self.client.workspace_folders().await {
            self.client
                .log_message(MessageType::INFO, format!("{:?}", workspace))
                .await;

            self.client
                .log_message(MessageType::INFO, "server initialized!!!!!")
                .await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        import_map: DashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

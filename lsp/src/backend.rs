use std::{collections::HashMap, fmt::Display};

use dashmap::{mapref::one::Ref, DashMap};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOptions, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, CreateFilesParams, DeleteFilesParams,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
        MessageType, Range, RenameFilesParams, ServerCapabilities, ServerInfo,
        TextDocumentIdentifier, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url,
        WorkDoneProgressOptions, WorkspaceEdit,
    },
    Client,
};

use super::TextDocument;

pub enum LogKind {
    ServerInitialized,
    FileOpened,
    FileClosed,
    FileChanged,
    FileSaved,
    FilesCreated,
    FilesRenamed,
    FilesDeleted,
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    files: DashMap<Url, TextDocument>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,
            files: DashMap::new(),
        }
    }
    pub async fn log(&self, kind: LogKind) {
        self.client
            .log_message(
                MessageType::INFO,
                match kind {
                    LogKind::ServerInitialized => "Server initialized",
                    LogKind::FileOpened => "File opened",
                    LogKind::FileClosed => "File closed",
                    LogKind::FileChanged => "File changed",
                    LogKind::FileSaved => "File saved",
                    LogKind::FilesCreated => "Files created",
                    LogKind::FilesDeleted => "Files deleted",
                    LogKind::FilesRenamed => "FilesRenamed",
                },
            )
            .await;
    }
    pub fn get_text_from_range(
        &self,
        text_document: TextDocumentIdentifier,
        range: Range,
    ) -> Option<(String, Ref<Url, TextDocument>)> {
        self.files
            .get(&text_document.uri)
            .map(|file| (file.select_range(&range).to_string(), file))
    }

    pub fn create_preferred_code_action(
        &self,
        title: impl Display,
        changes: HashMap<Url, Vec<TextEdit>>,
    ) -> CodeActionOrCommand {
        CodeActionOrCommand::CodeAction(CodeAction {
            title: title.to_string(),
            is_preferred: Some(true),
            kind: Some(CodeActionKind::QUICKFIX),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

impl Backend {
    pub async fn on_initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
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
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            ..Default::default()
        };

        Ok(InitializeResult {
            server_info,
            capabilities,
        })
    }

    pub async fn on_initialized(&self, params: InitializedParams) {}
    pub async fn on_open(&self, params: DidOpenTextDocumentParams) {
        self.files.insert(
            params.text_document.uri.clone(),
            TextDocument::new(params.text_document),
        );
    }
    pub async fn on_close(&self, params: DidCloseTextDocumentParams) {}
    pub async fn on_save(&self, params: DidSaveTextDocumentParams) {}
    pub async fn on_change(&self, params: DidChangeTextDocumentParams) {
        self.files.alter(&params.text_document.uri, |_, file| {
            file.commit(params.content_changes, params.text_document.version)
        });
    }
    pub async fn on_change_configuration(&self, params: DidChangeConfigurationParams) {}
    pub async fn on_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {}
    pub async fn on_create_files(&self, params: CreateFilesParams) {}
    pub async fn on_rename_files(&self, params: RenameFilesParams) {}
    pub async fn on_delete_files(&self, params: DeleteFilesParams) {}
    pub async fn on_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}
    pub async fn trigger_code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let Some((text_selected, file)) = self.get_text_from_range(params.text_document, params.range) else {
            return Ok(None)
        };

        let mut code_actions = vec![];

        if text_selected == "hello" {
            code_actions.push(self.create_preferred_code_action(
                "Change into hello twice",
                HashMap::from([(file.get_uri(), Vec::new())]),
            ));
        }

        if code_actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(code_actions))
        }
    }

    pub async fn on_code_action_resolve(&self, mut param: CodeAction) -> Result<CodeAction> {
        param.is_preferred = Some(true);
        Result::Ok(param)
    }
}

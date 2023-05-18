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
    Client, LanguageServer,
};

use super::TextDocument;

enum LogKind {
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
    async fn log(&self, kind: LogKind) {
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

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.on_initialize(params).await
    }

    async fn initialized(&self, params: InitializedParams) {
        self.on_initialized(params).await;
        self.log(LogKind::ServerInitialized).await
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_open(params).await;
        self.log(LogKind::FileOpened).await
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.on_close(params).await;
        self.log(LogKind::FileClosed).await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.on_save(params).await;
        self.log(LogKind::FileSaved).await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.log(LogKind::FileChanged).await;
        self.on_change(params).await;
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.on_change_configuration(params).await
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        self.on_change_workspace_folders(params).await
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        self.log(LogKind::FilesCreated).await;
        self.on_create_files(params).await
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        self.log(LogKind::FilesRenamed).await;
        self.on_rename_files(params).await
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        self.log(LogKind::FilesDeleted).await;
        self.on_delete_files(params).await
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.on_change_watched_files(params).await
    }

    async fn code_action_resolve(&self, param: CodeAction) -> Result<CodeAction> {
        self.on_code_action_resolve(param).await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.trigger_code_action(params).await
    }
}

impl Backend {
    async fn on_initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
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

    async fn on_initialized(&self, params: InitializedParams) {}
    async fn on_open(&self, params: DidOpenTextDocumentParams) {
        self.files.insert(
            params.text_document.uri.clone(),
            TextDocument::new(params.text_document),
        );
    }
    async fn on_close(&self, params: DidCloseTextDocumentParams) {}
    async fn on_save(&self, params: DidSaveTextDocumentParams) {}
    async fn on_change(&self, params: DidChangeTextDocumentParams) {
        self.files.alter(&params.text_document.uri, |_, file| {
            file.commit(params.content_changes, params.text_document.version)
        });
    }
    async fn on_change_configuration(&self, params: DidChangeConfigurationParams) {}
    async fn on_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {}
    async fn on_create_files(&self, params: CreateFilesParams) {}
    async fn on_rename_files(&self, params: RenameFilesParams) {}
    async fn on_delete_files(&self, params: DeleteFilesParams) {}
    async fn on_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}
    async fn trigger_code_action(
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

    async fn on_code_action_resolve(&self, mut param: CodeAction) -> Result<CodeAction> {
        param.is_preferred = Some(true);
        Result::Ok(param)
    }
}

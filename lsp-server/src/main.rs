use backend::{Backend, LogKind};
use document::TextDocument;
use tower_lsp::{LanguageServer, LspService, Server};

use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CodeAction, CodeActionParams, CodeActionResponse, CreateFilesParams, DeleteFilesParams,
        DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
        DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
        RenameFilesParams,
    },
};

mod backend;
mod document;

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

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}

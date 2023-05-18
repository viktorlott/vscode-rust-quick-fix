use backend::Backend;
use document::TextDocument;
use tower_lsp::{LspService, Server};

mod backend;
mod document;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}

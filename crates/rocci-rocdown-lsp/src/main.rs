use std::error::Error;

use lsp_server::{Connection, Message};
use lsp_types::InitializeParams;
use rocci_lsp::ProfiledRocBackend;
use rocci_rocdown_lsp::composed_server;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();
    let (id, params) = connection.initialize_start()?;
    let params: InitializeParams = serde_json::from_value(params)?;
    let mut server = composed_server();
    let trusted = params
        .initialization_options
        .as_ref()
        .and_then(|options| options["trustedWorkspace"].as_bool())
        .unwrap_or_else(|| std::env::var("ROCCI_LSP_TRUSTED_WORKSPACE").as_deref() == Ok("1"));
    server.set_roc_backend(Box::new(ProfiledRocBackend::new(trusted)));
    let result = server.initialize(params);
    connection.initialize_finish(id, serde_json::to_value(result)?)?;

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                connection
                    .sender
                    .send(Message::Response(server.handle_request(req)))?;
            }
            Message::Notification(not) => {
                if let Some(outgoing) = server.handle_notification(not) {
                    connection.sender.send(Message::Notification(outgoing))?;
                }
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

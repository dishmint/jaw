mod diagnostics;
mod goto;
mod hover;
mod rpc;
mod server;

use std::io::{self, BufReader};

fn main() {
    eprintln!("jaw-lsp: starting");

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    let mut server = server::Server::new();
    server.run(&mut reader, &mut writer);

    eprintln!("jaw-lsp: exiting");
}

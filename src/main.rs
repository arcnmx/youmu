#![feature(result_expect)]

extern crate rustc_serialize;
extern crate semver;
extern crate docopt;
extern crate iron;
extern crate router;
extern crate hyper;
extern crate cargo;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rusqlite;
extern crate handlebars;
extern crate handlebars_iron;
extern crate urlencoded;
extern crate tempdir;
extern crate mount;
extern crate staticfile;

mod server;
mod crates;
mod sqlite;
mod docs;

use docopt::Docopt;
use server::{Server, ServerConfig};
use crates::Cargo;
use sqlite::SqliteDocs;

#[derive(Clone, RustcDecodable, Debug)]
struct Args {
	flag_host: String,
	flag_port: u16,
	flag_docs: String,
}

const USAGE: &'static str = "
Usage:
	youmu [options]

Serves rust documentation.

Options:
	-h, --help          Show this message.
	-h, --host HOST     The local host to bind to [default: 0.0.0.0].
	-p, --port PORT     The local port to bind to [default: 8000].
	--docs PATH         The directory to store generated docs in [default: ./docs].
";

fn main() {
	if let Err(err) = env_logger::init() {
		println!("unable to log: {:?}", err);
	}

	let args: Args = Docopt::new(USAGE).and_then(|d| d.help(true).decode()).unwrap_or_else(|e| e.exit());
	let config = ServerConfig {
		bind_address: args.flag_host,
		bind_port: args.flag_port,
		docs_path: From::from(args.flag_docs),
	};

	let gen = Cargo::new();
	let docs = SqliteDocs::new();

	let (server, chain) = Server::new(config, docs, gen);
	let listen = server.listen(chain);

	listen.expect("Could not start server");
}

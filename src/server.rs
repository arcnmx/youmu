use std::sync::{Arc, RwLock, Mutex};
use std::net::{SocketAddr, ToSocketAddrs};
use std::any::Any;
use std::hash::{Hash, Hasher, SipHasher};
use std::path::PathBuf;
use std::fs::create_dir_all;
use iron::{Iron, Chain, Protocol, IronError, IronResult, Request, Response, Plugin, method, status};
use router::Router;
use iron::error::HttpResult;
use hyper::server::Listening;
use docs::{Docs, Docgen, PackageRequest, PackageSource};
use handlebars::Handlebars;
use handlebars_iron::{HandlebarsEngine, Template};
use urlencoded::UrlEncodedBody;
use rustc_serialize::json::{Json, ToJson, encode};
use rustc_serialize::Encodable;
use cargo::core::source::SourceId;
use cargo::util::human;
use semver::Version;

pub struct ServerConfig {
	pub bind_address: String,
	pub bind_port: u16,
	pub docs_path: PathBuf,
}

pub struct Server<D, G> {
	config: ServerConfig,
	docs: D,
	gen: Mutex<G>,
}

struct EncodeJson<T>(T);

impl<T: Encodable> ToJson for EncodeJson<T> {
	fn to_json(&self) -> Json {
		Json::from_str(&encode(&self.0).unwrap()).unwrap()
	}
}

const HANDLEBARS_INDEX_VALUE: &'static str = include_str!("../templates/index.html");
const HANDLEBARS_INDEX: &'static str = "index";

impl<D: Docs + Sync + Send + Any, G: Docgen + Sync + Send + Any> Server<D, G> {
	pub fn new(config: ServerConfig, docs: D, gen: G) -> (Arc<Self>, Chain) {
		let server = Arc::new(Server {
			config: config,
			docs: docs,
			gen: Mutex::new(gen),
		});

		let mut router = Router::new();

		router.route(method::Get, "/", {
			let server = server.clone();
			move |r: &mut Request| server.index_endpoint(r)
		});

		router.route(method::Post, "/gendocs", {
			let server = server.clone();
			move |r: &mut Request| server.gendocs_endpoint(r)
		});

		router.route(method::Get, "/docs/:crate", {
			let server = server.clone();
			move |r: &mut Request| server.index_endpoint(r)
		});

		router.route(method::Get, "/docs/:crate/:version", {
			let server = server.clone();
			move |r: &mut Request| server.index_endpoint(r)
		});

		let mut chain = Chain::new(router);

		let mut templates = Handlebars::new();
		templates.register_template_string(HANDLEBARS_INDEX, HANDLEBARS_INDEX_VALUE.into()).unwrap();

		chain.link_after(HandlebarsEngine {
			prefix: String::new(),
			suffix: String::new(),
			registry: RwLock::new(Box::new(templates)),
		});

		(server, chain)
	}

	pub fn listen(&self, chain: Chain) -> HttpResult<Listening> {
		Iron::new(chain).listen_with(self.address().expect("Unable to get bind address"), 4, Protocol::Http)
	}

	pub fn address(&self) -> Option<SocketAddr> {
		let config = self.config();
		let addr = format!("{}:{}", config.bind_address, config.bind_port);
		(&addr[..]).to_socket_addrs().ok().and_then(|mut addrs| addrs.next())
	}

	pub fn config(&self) -> &ServerConfig {
		&self.config
	}

	fn index_endpoint(&self, _req: &mut Request) -> IronResult<Response> {
		let crates = try!(self.docs.query_all().map_err(|e| IronError::new(e, status::InternalServerError)));

		#[derive(RustcEncodable)]
		struct Version {
			href: String,
			version: String,
		}

		#[derive(RustcEncodable)]
		struct Crate {
			href: String,
			name: String,
			versions: Vec<Version>,
		}

		#[derive(RustcEncodable)]
		struct Index {
			total: usize,
			crates: Vec<Crate>,
			request_action: &'static str,
		}

		Ok(Response::with((
			status::Ok,
			Template::new(HANDLEBARS_INDEX, EncodeJson(Index {
				total: crates.len(),
				crates: crates.into_iter().map(|c| Crate {
					href: format!("docs/{}", c.name),
					name: c.name,
					versions: Vec::new(),
				}).collect::<Vec<_>>(),
				request_action: "gendocs",
			})),
		)))
	}

	fn gendocs_endpoint(&self, req: &mut Request) -> IronResult<Response> {
		let args = try!(req.get_ref::<UrlEncodedBody>().map_err(|e| IronError::new(e, status::BadRequest)));

		let get_arg = |key: &'static str| args.get(key).and_then(|v| v.get(0)).and_then(|v| if v.len() > 0 { Some(v) } else { None });


		let request = PackageRequest {
			name: try!(get_arg("package").ok_or_else(|| IronError::new(human("expected package name"), status::BadRequest))).clone(),
			source: if let Some(url) = get_arg("url") {
				PackageSource::Url(url.clone())
			} else if let Some(version) = get_arg("version") {
				PackageSource::CratesIO(try!(Version::parse(version).map_err(|e| IronError::new(e, status::BadRequest))))
			} else {
				return Err(IronError::new(human("expected version or url"), status::BadRequest))
			},
			features: Vec::new(),
			default_features: true,
			include_deps: true,
		};

		let version = match request.source {
			PackageSource::CratesIO(ref version) => version.to_string(),
			PackageSource::Url(ref url) => {
				let mut hasher = SipHasher::new();
				SourceId::from_url(url.clone()).hash(&mut hasher);
				format!("{:016x}", hasher.finish())
			},
		};

		let mut dest = self.config.docs_path.clone();
		dest.push(&request.name);
		dest.push(version);
		try!(create_dir_all(&dest).map_err(|e| IronError::new(e, status::InternalServerError)));

		try!(self.gen.lock().unwrap().document(&request, dest).map_err(|e| IronError::new(e, status::InternalServerError)));

		Ok(Response::with((status::Ok)))
	}
}

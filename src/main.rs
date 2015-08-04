extern crate rustc_serialize;
extern crate semver;
extern crate docopt;
extern crate cargo;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tempdir;
extern crate walker;
extern crate yaml_rust as yaml;

mod crates;
mod docs;

use std::path::PathBuf;
use std::error::Error;
use std::io::{self, Read};
use std::fs::{File, create_dir_all};
use docopt::Docopt;
use yaml::{YamlLoader, Yaml};
use semver::VersionReq;
use crates::Cargo;
use docs::{Docgen, PackageRequest, PackageSource};

#[derive(Clone, RustcDecodable, Debug)]
struct Args {
    flag_out: String,
    flag_ver: String,
    flag_url: String,
    flag_features: String,
    flag_no_default_features: bool,
    flag_no_deps: bool,
    arg_path: String,
    arg_package: String,
    cmd_doc: bool,
    cmd_konpaku: bool,
}

const USAGE: &'static str = "
Usage:
    youmu doc [options] <package>
    youmu konpaku [options] <path>
    youmu --help

Generates rust documentation.

Options:
    -h, --help              Show this message.
    -o PATH, --out PATH     The directory to store the output in [default: ./docs].
    --ver VERSION           Package version requirement [default: *].
    --url URL               URL for package repository.
    --no-default-features   Don't set default package features.
    --features FEATURES     Space-separated list of features to build with.
    --no-deps               Inhibit documentation of dependencies.
";

fn main() {
    if let Err(err) = env_logger::init() {
        println!("unable to log: {:?}", err);
    }

    let args = Docopt::new(USAGE).and_then(|d| d.help(true).decode()).unwrap_or_else(|e| e.exit());

    main_impl(args).unwrap();
}

fn main_impl(args: Args) -> io::Result<()> {
    match args {
        Args { cmd_doc: true, arg_package, flag_out, flag_ver, flag_url, flag_no_default_features, flag_no_deps, flag_features, .. } => {
            let request = PackageRequest {
                name: arg_package.into(),
                source: if !flag_url.is_empty() {
                    PackageSource::Url(flag_url)
                } else {
                    PackageSource::CratesIo(try!(VersionReq::parse(&flag_ver[..]).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))))
                },
                features: flag_features.split(' ').map(Into::into).collect(),
                default_features: !flag_no_default_features,
                include_deps: !flag_no_deps,
            };
            try!(create_dir_all(&flag_out));
            try!(Cargo::new().document(&request, flag_out).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.description())));
        },
        Args { cmd_konpaku: true, arg_path, flag_out, .. } => {
            fn invalid_input(msg: &'static str) -> io::Error {
                io::Error::new(io::ErrorKind::InvalidInput, msg)
            }

            let mut gen = Cargo::new();

            let mut data = String::new();
            let mut f = try!(File::open(arg_path));
            try!(f.read_to_string(&mut data));
            let data = try!(YamlLoader::load_from_str(&data[..]).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e)));
            let data = try!(data.get(0).and_then(Yaml::as_vec).ok_or_else(|| invalid_input("expected root array")));
            for entry in data {
                let entry = try!(entry.as_hash().ok_or_else(|| invalid_input("expected hash")));
                let request = PackageRequest {
                    name: try!(entry.get(&Yaml::String("package".into())).and_then(Yaml::as_str).ok_or_else(|| invalid_input("expected package name"))).into(),
                    source: if let Some(url) = entry.get(&Yaml::String("url".into())).and_then(Yaml::as_str) {
                        PackageSource::Url(url.into())
                    } else if let Some(version) = entry.get(&Yaml::String("version".into())).and_then(Yaml::as_str) {
                        PackageSource::CratesIo(try!(VersionReq::parse(version).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))))
                    } else {
                        PackageSource::CratesIo(VersionReq::any())
                    },
                    features: entry.get(&Yaml::String("features".into())).and_then(Yaml::as_vec).map(|v| v.into_iter().filter_map(Yaml::as_str).map(Into::into).collect()).unwrap_or(Vec::new()),
                    default_features: entry.get(&Yaml::String("default-features".into())).and_then(Yaml::as_bool).unwrap_or(true),
                    include_deps: entry.get(&Yaml::String("include-deps".into())).and_then(Yaml::as_bool).unwrap_or(false),
                };
                let mut out = PathBuf::from(&flag_out);
                out.push(&request.name);
                try!(gen.document(&request, out).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.description())));
            }
        },
        _ => docopt::Error::WithProgramUsage(Box::new(docopt::Error::Help), USAGE.into()).exit(),
    }

    Ok(())
}

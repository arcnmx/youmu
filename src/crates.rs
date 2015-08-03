use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, remove_dir_all};
use std::env::{current_dir, set_var};
use std::os::unix::fs::symlink;
use cargo::core::shell::{MultiShell, Shell, ShellConfig, Verbosity};
use cargo::util::config::Config;
use cargo::util::errors::CargoError;
use cargo::core::source::{Source, SourceId};
use cargo::core::package_id::PackageId;
use cargo::core::registry::Registry;
use cargo::core::dependency::Dependency;
use cargo::ops::{CompileOptions, CompileFilter, CompileMode, compile_pkg};
use cargo::sources::registry::RegistrySource;
use cargo::human;
use std::io::sink;
use docs::{Docgen, PackageRequest, PackageSource};
use semver::VersionReq;
use tempdir::TempDir;

pub struct Cargo;

impl Cargo {
	pub fn new() -> Self {
		Cargo
	}

	fn config() -> Config {
		use std::io::{stdout, stderr};
		//let verbosity = Verbosity::Quiet;
		let verbosity = Verbosity::Verbose;
		let config = ShellConfig {
			color: true,
			verbosity: verbosity,
			tty: false,
		};
		//let out = Shell::create(Box::new(sink()) as Box<_>, config);
		//let err = Shell::create(Box::new(sink()) as Box<_>, config);
		let out = Shell::create(Box::new(stdout()) as Box<_>, config);
		let err = Shell::create(Box::new(stderr()) as Box<_>, config);
		let shell = MultiShell::new(out, err, verbosity);
		Config::new(shell).unwrap()
	}
}

impl Docgen for Cargo {
	type Error = Box<CargoError>;

	fn document<P: AsRef<Path>>(&mut self, r: &PackageRequest, dest: P) -> Result<(), Self::Error> {
		let dir = try!(TempDir::new("youmu_cargo"));
		set_var("CARGO_TARGET_DIR", dir.path());
		try!(create_dir_all(dir.path()));

		let config = &Self::config();

		let (mut source, pkg) = match r.source {
			PackageSource::CratesIO(ref version) => {
				let registry = try!(SourceId::for_central(config));
				let source = RegistrySource::new(&registry, config);
				let pkg = try!(PackageId::new(&r.name[..], version, &registry));
				(Box::new(source) as Box<_>, pkg)
			},
			PackageSource::Url(ref url) => {
				let registry = SourceId::from_url(url.clone());
				let mut source = registry.load(config);
				let summary = try!(source.query(&try!(Dependency::parse(&r.name[..], None, &registry))));
				let summary = try!(summary.get(0).ok_or_else(|| human("unable to determine version")));
				(source, summary.package_id().clone())
			},
		};

		try!(source.update());
		try!(source.download(&[pkg.clone()]));
		let pkg = try!(source.get(&[pkg])).into_iter().next();
		let pkg = try!(pkg.ok_or_else(|| human("unable to find package")));

		let mut path = PathBuf::from(config.target_dir(&pkg));
		path.push("doc");
		let _ = remove_dir_all(&path);
		try!(symlink(try!(current_dir()).join(dest.as_ref()), &path));

		let options = CompileOptions {
			config: config,
			jobs: None,
			target: None,
			features: &r.features[..],
			no_default_features: !r.default_features,
			spec: None,
			filter: CompileFilter::Everything,
			exec_engine: None,
			release: false,
			mode: CompileMode::Doc {
				deps: r.include_deps,
			},
			target_rustc_args: None,
		};
		//let mut path_source = try!(PathSource::for_path(pkg.manifest_path().parent().unwrap(), &config));
		//try!(path_source.update());
		//try!(compile_pkg(&pkg, Some(source), &options));
		try!(compile_pkg(&pkg, None, &options));

		Ok(())
	}
}

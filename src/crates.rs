use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::env::{current_dir, set_var, var_os};
use std::os::unix::fs::symlink;
use cargo::core::shell::{MultiShell, Shell, ShellConfig, Verbosity};
use cargo::util::config::Config;
use cargo::util::errors::CargoError;
use cargo::core::source::{Source, SourceId};
use cargo::core::registry::Registry;
use cargo::core::dependency::Dependency;
use cargo::core::summary::Summary;
use cargo::ops::{CompileOptions, CompileFilter, CompileMode, compile_pkg};
use cargo::sources::registry::RegistrySource;
use cargo::human;
use docs::{Docgen, PackageRequest, PackageSource};
use tempdir::TempDir;
use walker::Walker;

pub struct Cargo;

impl Cargo {
    pub fn new() -> Self {
        Cargo
    }

    fn config() -> Config {
        use std::io::{stdout, stderr};
        let verbosity = Verbosity::Normal;
        let config = ShellConfig {
            color: true,
            verbosity: verbosity,
            tty: true,
        };
        let out = Shell::create(Box::new(stdout()) as Box<_>, config);
        let err = Shell::create(Box::new(stderr()) as Box<_>, config);
        let shell = MultiShell::new(out, err, verbosity);
        Config::new(shell).unwrap()
    }
}

impl Docgen for Cargo {
    type Error = Box<CargoError>;

    fn document<P: AsRef<Path>>(&mut self, r: &PackageRequest, dest: P) -> Result<(), Self::Error> {
        let _guard = if var_os("CARGO_TARGET_DIR").is_none() {
            let guard = try!(TempDir::new("youmu_docs"));
            set_var("CARGO_TARGET_DIR", guard.path());
            Some(guard)
        } else {
            None
        };
        let config = &Self::config();

        let (mut source, pkg) = match r.source {
            PackageSource::CratesIo(ref version) => {
                let registry = try!(SourceId::for_central(config));
                let mut source = RegistrySource::new(&registry, config);
                try!(source.update());
                let summary = try!(source.query(&try!(Dependency::parse(&r.name[..], None, &registry)).set_version_req(version.clone())));
                let summary = try!(summary.into_iter().fold(None, |prev: Option<Summary>, next| Some(if let Some(prev) = prev {
                    if prev.version() < next.version() {
                        next
                    } else {
                        prev
                    }
                } else {
                    next
                })).ok_or_else(|| human("unable to find specified version")));
                (Box::new(source) as Box<_>, summary.package_id().clone())
            },
            PackageSource::Url(ref url) => {
                let registry = SourceId::from_url(url.clone());
                let mut source = registry.load(config);
                try!(source.update());
                let summary = try!(source.query(&try!(Dependency::parse(&r.name[..], None, &registry))));
                let summary = try!(summary.get(0).ok_or_else(|| human("unable to determine version")));
                (source, summary.package_id().clone())
            },
        };

        try!(source.download(&[pkg.clone()]));
        let pkg = try!(source.get(&[pkg])).into_iter().next();
        let pkg = try!(pkg.ok_or_else(|| human("unable to find package")));

        let mut path = PathBuf::from(config.target_dir(&pkg));
        try!(create_dir_all(&path));
        if let Ok(walker) = Walker::new(path.join("debug/.fingerprint").as_path()) {
            for entry in walker {
                let entry = try!(entry);
                if entry.file_name().to_str().map(|v| v.starts_with("doc-")).unwrap_or(false) {
                    try!(remove_file(entry.path()));
                }
            }
        }
        let dest = try!(current_dir()).join(dest.as_ref());
        try!(create_dir_all(&dest));
        path.push("doc");
        let _ = remove_file(&path);
        let _ = remove_dir_all(&path);
        try!(symlink(dest, &path));

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
        try!(compile_pkg(&pkg, None, &options));

        Ok(())
    }
}

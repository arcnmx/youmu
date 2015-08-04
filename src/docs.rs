use std::error::Error;
use std::path::Path;
use semver::VersionReq;

pub enum PackageSource {
    CratesIo(VersionReq),
    Url(String),
}

pub struct PackageRequest {
    pub name: String,
    pub source: PackageSource,
    pub features: Vec<String>,
    pub default_features: bool,
    pub include_deps: bool,
}

pub trait Docgen {
    type Error: Error + Send + 'static;

    fn document<P: AsRef<Path>>(&mut self, r: &PackageRequest, dest: P) -> Result<(), Self::Error>;
}

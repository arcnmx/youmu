use std::error::Error;
use std::path::Path;
use semver::Version;

pub enum PackageSource {
	CratesIO(Version),
	Url(String),
}

pub struct PackageRequest {
	pub name: String,
	pub source: PackageSource,
	pub features: Vec<String>,
	pub default_features: bool,
	pub include_deps: bool,
}

pub struct PackageDocs {
	pub name: String,
	pub version: Version,
	pub features: Vec<String>,
}

pub trait Docgen {
	type Error: Error + Send + 'static;

	fn document<P: AsRef<Path>>(&mut self, r: &PackageRequest, dest: P) -> Result<(), Self::Error>;
}

pub trait Docs {
	type Error: Error + Send + 'static;

	fn query_all(&self) -> Result<Vec<PackageDocs>, Self::Error>;
}

use rusqlite::SqliteError;
use semver::Version;
use docs::{Docs, PackageDocs};

pub struct SqliteDocs;

impl Docs for SqliteDocs {
	type Error = SqliteError;

	fn query_all(&self) -> Result<Vec<PackageDocs>, Self::Error> {
		Ok(vec![
			PackageDocs {
				name: "youmu".into(),
				version: Version::parse("0.0.1").unwrap(),
				features: Vec::new(),
			},
		])
	}
}

impl SqliteDocs {
	pub fn new() -> Self {
		SqliteDocs
	}
}

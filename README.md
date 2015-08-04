# youmu

[![travis-badge][]][travis] [![license-badge][]][license]

Documents Rust packages from [crates.io](https://crates.io) and [GitHub](https://github.com).

http://arcnmx.github.io/youmu-docs/

## Generating Documentation

[Edit this file](https://github.com/arcnmx/youmu-docs/edit/master/crates.yml) by forking
the [youmu-docs](https://github.com/arcnmx/youmu-docs) repo and issuing a pull request.
The documentation for the requested crate will be automatically generated for you. You may
delete your fork as soon as the pull request is made.

### crates.yml

This file lists all the crates to generate documentation for - which can be edited, added,
or removed at will. See the example below for the more advanced generation options.

```yaml
- package: hyper
  version: ^0.6
  default-features: false
  features:
      - ssl
  include-deps: false

- package: rustc-serialize
  url: git+https://github.com/rust-lang/rustc-serialize
```

[travis-badge]: https://img.shields.io/travis/arcnmx/youmu/master.svg?style=flat-square
[travis]: https://travis-ci.org/arcnmx/youmu
[license-badge]: https://img.shields.io/badge/license-MIT-lightgray.svg?style=flat-square
[license]: https://github.com/arcnmx/youmu/blob/master/COPYING

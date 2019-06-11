# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.3.2] - 2019-06-11

### Changed

* Derive `Debug` for `client::download::Download`.
* `client::download::Download` uses references.


## [0.3.1] - 2019-06-06

### Changed

* Derive `Debug` for `client::upload::Upload`.


## [0.3.0] - 2019-06-06

### Added

* Exporting `reqwest::IntoUrl` because it's part of the public API in `auth::CodeProvider`.
* `auth::Token` can be cloned.

### Changed

* `ClientCredentials` uses references
* `UnauthorizedClient` and `AuthorizedClient` use references


## [0.2.0] - 2019-06-06

### Changed

* Simplified errors


## [0.1.0] - 2019-06-03

Initial release supports
* auth
* search
* upload
* delete
* download

[Unreleased]: https://github.com/lukaspustina/ceres/compare/v0.3.2...HEAD
[0.3.2]: https://github.com/lukaspustina/ceres/compare/v0.3.1...0.3.2
[0.3.1]: https://github.com/lukaspustina/ceres/compare/v0.3.0...0.3.1
[0.3.0]: https://github.com/lukaspustina/ceres/compare/v0.2.0...0.3.0
[0.2.0]: https://github.com/lukaspustina/ceres/compare/v0.1.0...0.2.0
[0.1.0]: https://github.com/lukaspustina/ceres/compare/v0.0.1...0.1.0


# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]


### Fixes
* Collection results in-accessible because of private visibility
* Send parameters as query instead of form parameters in collection search

## [0.3.8] - 2019-06-13

### Add

* Users listing
* Collections listing

### Change

* search documents returns empty `Vec` instead of None in case no documents have been found.


## [0.3.7] - 2019-06-13

### Add

* General error handling

### Fixed

* Optional fields in search results


## [0.3.6] - 2019-06-12

### Add

* Serialization to search and delete results


## [0.3.5] - 2019-06-12

### Changed

* struct visibilities changed


## [0.3.4] - 2019-06-12

### Changed

* Enhances search results by datetime, representations, and extented-metadata
* `Search` derives `Debug`


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

[Unreleased]: https://github.com/lukaspustina/ceres/compare/v0.3.8...HEAD
[0.3.8]: https://github.com/lukaspustina/ceres/compare/v0.3.7...0.3.8
[0.3.7]: https://github.com/lukaspustina/ceres/compare/v0.3.6...0.3.7
[0.3.6]: https://github.com/lukaspustina/ceres/compare/v0.3.5...0.3.6
[0.3.5]: https://github.com/lukaspustina/ceres/compare/v0.3.4...0.3.5
[0.3.4]: https://github.com/lukaspustina/ceres/compare/v0.3.3...0.3.4
[0.3.3]: https://github.com/lukaspustina/ceres/compare/v0.3.2...0.3.3
[0.3.2]: https://github.com/lukaspustina/ceres/compare/v0.3.1...0.3.2
[0.3.1]: https://github.com/lukaspustina/ceres/compare/v0.3.0...0.3.1
[0.3.0]: https://github.com/lukaspustina/ceres/compare/v0.2.0...0.3.0
[0.2.0]: https://github.com/lukaspustina/ceres/compare/v0.1.0...0.2.0
[0.1.0]: https://github.com/lukaspustina/ceres/compare/v0.0.1...0.1.0


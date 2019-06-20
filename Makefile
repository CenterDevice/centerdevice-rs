all: check build test

todos:
	rg --vimgrep -g '!Makefile' -i todo 

check:
	cargo check --all --examples --tests --benches

build:
	cargo check --all --examples

test:
	cargo test --all --no-fail-fast

clean-package:
	cargo clean -p $$(cargo read-manifest | jq .name)

release: clean-package release-test release-bump all
	git commit -am "Bump to version $$(cargo read-manifest | jq .version)"
	git tag v$$(cargo read-manifest | jq -r .version)

release-test: check test clippy
	cargo fmt -- --check
	cargo publish --dry-run

release-bump:
	cargo bump

publish:
	git push && git push --tags
	cargo publish

clippy:
	cargo clippy --all --all-targets -- -D warnings $$(source ".clippy.args")

fmt:
	cargo fmt

duplicate_libs:
	cargo tree -d

_update-clippy_n_fmt:
	rustup update
	rustup component add clippy rustfmt

_cargo_install:
	cargo install -f cargo-tree
	cargo install -f cargo-bump

.PHONY: tests


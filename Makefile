.PHONY: release

release:
	cargo build --release

dev:
	cargo build
	mv target/debug/ub .

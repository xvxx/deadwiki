CARGO_DEBUG = cargo build
CARGO_RELEASE = cargo build --release
PREFIX=$(HOME)

.PHONY: watch
watch:
	cargo watch -i wiki/ -x 'run wiki/ -p 9999'

.PHONY: debug
debug: target/debug/dead
	cp $< .

target/debug/dead: src/*.rs
	$(CARGO_DEBUG)

.PHONY: release
release: target/release/dead

target/release/dead: src/*.rs
	$(CARGO_RELEASE)

clean:
	rm -rf target

install: release
	cp target/release/dead $(PREFIX)/bin

uninstall: release
	rm -f $(PREFIX)/bin/dead

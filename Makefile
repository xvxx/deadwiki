CARGO_DEBUG = cargo build
CARGO_RELEASE = cargo build --release

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
	cp $< .

target/release/dead: src/*.rs
	$(CARGO_RELEASE)

.PHONY: gui
gui:
	$(CARGO_RELEASE) --features gui
	cp target/release/dead .

clean:
	rm -f target/{release,debug}/dead dead

test:
	cargo test --all
	cargo test --all --release
	make test-wasm

lint:
	cargo clippy \
      -- \
      \
      -W clippy::all \
      -W clippy::pedantic \
      \
      -A clippy::missing_panics_doc \
      -A clippy::must_use_candidate \
      -A clippy::missing_errors_doc \
      \
      -D warnings

test-wasm:
	cargo install wasm-pack
	wasm-pack test --firefox --headless

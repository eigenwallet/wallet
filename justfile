# Show help for each of the justfile recipes
help:
	@just --list

# Clean the Monero C++ Codebase
clean-monero:
	rm -rf monero-sys/monero/
	just update-submodules

# Builds the Rust bindings for Monero
build-monero-sys: update-submodules	
	cd monero-sys && cargo build

# Test the FFI bindings using various sanitizers, that can detect memory safety issues.
test-ffi: test-ffi-address

# Tests the FFI bindings using AddressSanitizer. Can detect memory safety issues like use-after-free, double-free, leaks, etc.
test-ffi-address:
	just update-submodules
	cd monero-sys && RUSTFLAGS=-Zsanitizer=address cargo +nightly nextest run -Zbuild-std --target=`rustc --version --verbose | grep "host:" | cut -d' ' -f2`

# Start the Tauri app
tauri: update-submodules
	cd src-tauri && cargo tauri dev --no-watch -- -- --testnet

tauri-mainnet: update-submodules
	cd src-tauri && cargo tauri dev --no-watch

# Install the GUI dependencies
gui-install:
	cd src-gui && yarn install

# Start the GUI Dev Server
web: gui-install
	cd src-gui && yarn dev

# Start the app by starting the web server and the tauri app
gui:
	just web & just tauri

# Start the app by starting the web server and the tauri app on mainnet
gui-mainnet:
	just web & just tauri-mainnet

# Build the GUI
gui-build: gui-install
	cd src-gui && yarn build

# Run the Rust tests
tests: update-submodules
	cargo nextest run

# Tests the Rust bindings for Monero
test-monero-sys: update-submodules
	cd monero-sys && cargo nextest run
	just test-ffi

# Builds the ASB and Swap binaries
swap: update-submodules
	cd swap && cargo build --bin asb --bin=swap

# Run the asb on testnet
asb:
	cd swap && cargo run --bin asb -- --trace --testnet start

# Updates our submodules (currently only Monero C++ codebase)
update-submodules:
	cd monero-sys && git submodule update --init --recursive --force

# Run all the linting checks
lint: clippy lint-gui

# Run clippy checks
clippy: update-submodules
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Generate the bindings for the Tauri API
bindings: gui-install
	cd src-gui && yarn run gen-bindings

# Format the code
fmt:
	dprint fmt

# Run eslint for the GUI frontend
lint-gui-eslint: gui-install
	cd src-gui && yarn run eslint

# Run the typescript type checker for the GUI frontend
lint-gui-tsc: gui-install
	cd src-gui && yarn run tsc --noEmit

# Run the checks for the GUI frontend
lint-gui: gui-install
	just lint-gui-eslint || true
	just check-gui-tsc

# Sometimes you have to prune the docker network to get the integration tests to work
docker-prune-network:
	docker network prune -f

# Install dependencies required for building monero-sys
macos-dependencies:
	cd dev_scripts && chmod +x ./brew_dependencies_install.sh && ./brew_dependencies_install.sh

# Takes a crate (e.g monero-rpc-pool) and uses code2prompt to copy it's content to the clipboard.
prompt-crate crate:
	cd {{crate}} && code2prompt . --exclude "*.lock" --exclude ".sqlx/*" --exclude "target" --exclude "monero" --exclude "target-check"
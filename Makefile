.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9_-]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

#.PHONY: build_monero_cpp
#build_monero_cpp: # Build Monero C++ Codebase
#	make update_submodules && \
#	cd $(CURDIR)/monero-sys/monero && make -j8 release

.PHONY: clean_monero_cpp
clean_monero_cpp: # Clean the Monero C++ Codebase
	rm -rf $(CURDIR)/monero-sys/monero/ && \
	make update_submodules

.PHONY: build_monero_sys
monero_sys: # Builds the Rust bindings for Monero
	make update_submodules && \
	cd $(CURDIR)/monero-sys && cargo build

.PHONY: clean_monero_sys
tauri: # Start the Tauri app
	cd $(CURDIR)/src-tauri && cargo tauri dev --no-watch -- -- --testnet

.PHONY: gui_install
gui_install: # Install the GUI dependencies
	cd $(CURDIR)/src-gui && yarn install

.PHONY: gui_dev
gui: # Start the GUI Dev Server
	cd $(CURDIR)/src-gui && yarn dev

.PHONY: gui_build
gui_build: # Build the GUI
	cd $(CURDIR)/src-gui && yarn build

.PHONY: test_monero_sys
test_monero_sys: # Tests the Rust bindings for Monero
	cd $(CURDIR)/monero-sys && cargo test

.PHONY: swap
swap: # Builds the ASB and Swap binaries
	cd $(CURDIR)/swap && cargo build --bin asb --bin=swap

.PHONY: update_submodules
update_submodules: # Updates the our submodules (currently only Monero C++ codebase)
	cd $(CURDIR)/monero-sys && git submodule update --init --recursive --force

.PHONY: clippy_check
clippy_check: # Run clippy checks
	cargo clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: check_bindings
check_bindings: # Check the bindings for the Tauri API
	cd $(CURDIR)/src-gui && yarn run check-bindings

.PHONY: bindings
bindings: # Generate the bindings for the Tauri API
	cd $(CURDIR)/src-gui && yarn run gen-bindings

.PHONY: kill_monero_wallet_rpc
kill_monero_wallet_rpc: # Kill all instances of monero-wallet-rpc running in the background
	killall monero-wallet-rpc && pkill -f monero-wallet-rpc

.PHONY: fmt
fmt: # Format the code
	dprint fmt
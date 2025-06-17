# Latest Branch Changes Summary - XMR-BTC Swap

Based on the git history and changelog analysis for the `master` branch (cursor/summarize-latest-branch-changes-7033), here are the most significant recent changes:

## Recent Major Release: 2.2.0-beta (June 17, 2025)

### Most Recent Commits (Last 2 weeks):

1. **Release 2.2.0-beta (#416)** - *June 17, 2025*
   - Version bump and release preparation
   - Added VS Code settings configuration

2. **Monero Wallet Retry Logic (#417)** - *June 17, 2025*
   - Added retry logic to monero-sys for improved reliability
   - Enhanced error handling and logging for Monero wallet operations
   - Modified wallet synchronization and state management

3. **Major Monero Bindings Overhaul (#303)** - *June 17, 2025*
   - **BREAKING CHANGE**: Complete replacement of monero-wallet-rpc with direct FFI bindings
   - Introduced new `monero-sys` crate with C++ bindings to Monero core
   - Massive refactoring affecting 77 files with 5,646 additions and 2,243 deletions
   - Added comprehensive build system for Monero C++ integration

4. **ASB Early Refund Retry Logic (#412)** - *Earlier in June*
   - Added retry mechanism for early refund transactions
   - Improved swap reliability when early refunds fail

5. **Reliable Peer Discovery (#408)**
   - Enhanced peer discovery mechanisms
   - Improved network reliability and connection management

## Major Technical Changes in 2.2.0-beta:

### 🔄 Monero Integration Overhaul
- **Eliminated monero-wallet-rpc dependency** - Now uses direct FFI bindings to Monero core
- **New `export-monero-wallet` command** - Allows extraction of 25-word mnemonic seed and restore height
- **Breaking config change**: Removed `wallet_url`, added optional `daemon_url`
- **Migration required** for Docker compose setups to move wallet files between containers

### 🔧 Logging Changes
- **Logs now output to stderr** instead of stdout
- Scripts relying on log piping need to be updated (e.g., `asb logs 2>&1 | my-script.sh`)

### 🌐 Enhanced Peer Discovery
- **Multi-rendezvous support** - Can connect to multiple rendezvous points simultaneously
- **Local peer caching** - Remembers previously connected peers
- **Improved connection reliability**

### ⚡ ASB Improvements
- **6-hour retry window** for early refund transactions
- **Automatic abort** if Bob cancels the swap during retry period
- **Better error handling** and recovery mechanisms

## Recent Bug Fixes and Improvements:

### 2.0.x Series (June 2025):
- **Auto updater fixes** - Resolved issues with update prompts not displaying
- **Electrum load balancer improvements** - Increased timeouts and retry counts
- **Flatpak build support** - Added Flatpak packaging to release workflow
- **Tauri signature verification** - Added documentation for GUI signature verification

### Previous Notable Changes:
- **Early Bitcoin refund protocol** - Collaborative signing to avoid 12h timelock waits
- **Electrum load balancing** - Multiple server support for improved Bitcoin network reliability
- **MUI v7 upgrade** - Updated React Material-UI components
- **Conservative Monero fee calculation** - More accurate fee estimation for max swap amounts

## Development Infrastructure:

### CI/CD Improvements:
- **Large GitHub runners** for expensive builds (Monero compilation)
- **Aggressive caching strategies** including S3 caching
- **Parallel build optimizations**
- **Static linking** for better portability (OpenSSL, boost, protobuf)

### Build System Changes:
- **Rust 1.82 toolchain** requirement
- **CMake configuration** for Monero C++ compilation
- **Cross-platform build fixes** for macOS, Linux, and Windows
- **Docker optimizations** for faster builds

## Migration Notes:

### For ASB Operators:
1. **Wallet Migration**: Copy wallet files from monero-wallet-rpc volume to ASB volume
2. **Config Update**: Replace `wallet_url` with optional `daemon_url` 
3. **Log Parsing**: Update scripts to capture stderr instead of stdout
4. **Export Capability**: Use new `export-monero-wallet` command to backup seeds

### For Developers:
1. **API Changes**: Monero wallet API completely rewritten with FFI bindings
2. **Error Handling**: New exception handling for C++ FFI calls
3. **Build Dependencies**: Additional system dependencies for Monero compilation
4. **Testing**: Updated test harness to work with new Monero integration

This represents one of the most significant architectural changes in the project's history, moving from RPC-based Monero interaction to direct FFI bindings for improved performance and reliability.
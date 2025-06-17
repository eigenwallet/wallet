# Recent Changes to UnstoppableSwap/core/master

## Overview

UnstoppableSwap is a decentralized atomic swap implementation that enables trustless exchanges between Bitcoin and Monero. The project has recently undergone significant development activity, with a major release (2.2.0-beta) and several important improvements.

## Latest Major Release: 2.2.0-beta (June 17, 2025)

### Revolutionary Changes
The 2.2.0-beta release introduces **major architectural changes** that fundamentally alter how the swap protocol operates:

#### 1. Direct Monero Integration via FFI Bindings
- **Eliminated `monero-wallet-rpc` dependency**: The application now calls Monero functions directly through FFI (Foreign Function Interface) bindings instead of communicating with the external `monero-wallet-rpc` process
- **Breaking change for ASB users**: Monero wallets are no longer accessible by connecting to `monero-wallet-rpc`
- **Migration required**: Users running the ASB (Automated Swap Backend) with Docker must migrate wallet files from the `monero-wallet-rpc` container to the ASB container

#### 2. Configuration Changes
- **Removed `wallet_url` option**: Replaced with optional `daemon_url` parameter that specifies which Monero node the ASB connects to
- **Automatic node selection**: If `daemon_url` is not specified, the ASB connects to a random public Monero node

#### 3. New Wallet Management Features
- **Export command**: New `export-monero-wallet` command provides the Monero wallet's seed (25-word mnemonic) and restore height
- **External wallet compatibility**: Users can import the seed into external wallet software to manage Monero funds independently

## Recent Commits and Features (Since December 2024)

### Major Features Added

#### 1. **Retry Logic for Monero Wallet** (cf669a87)
- Enhanced reliability for Monero wallet operations
- Automatic retry mechanisms for failed wallet interactions

#### 2. **Monero FFI Bindings** (2e6d324a) 
- Core implementation of the direct Monero integration mentioned above
- Replaces RPC-based communication with direct function calls

#### 3. **Enhanced Early Refund System** (26eaf06e)
- ASB now retries `tx_early_refund` transactions for 6 hours before giving up
- Improved handling when Bob cancels a swap - Alice now properly aborts and lets Bob refund himself

#### 4. **Reliable Peer Discovery** (4702bd5b)
- Multiple rendezvous point connections simultaneously
- Local caching of previously connected peers
- Improved reconnection to cached peers even when not registered with rendezvous points

#### 5. **Flatpak Build Support** (757a3774)
- Added support for Flatpak package distribution
- Better Linux desktop integration

### Network and Infrastructure Improvements

#### 1. **Enhanced Electrum Integration**
- Increased `request_timeout` to 15s and `min_retries` to 15 for better reliability
- Load balancing across multiple Electrum servers
- Parallel transaction broadcasting to all servers

#### 2. **Logging Improvements**
- Logs now written to `stderr` instead of `stdout`
- Better separation of log streams for automated scripts

### Version History (Recent Releases)

#### 2.0.3 (June 12, 2025)
- GUI: Fixed auto updater display issue  
- Increased Electrum timeout settings for better reliability

#### 2.0.2 (June 12, 2025)
- Auto updater fixes
- Electrum load balancer improvements

#### 2.0.0 (June 12, 2025) - Major Protocol Update
- **Breaking protocol change**: v2.0.0+ cannot initiate swaps with pre-2.0.0 versions
- **Early Bitcoin refund feature**: Collaborative signing of `tx_refund_early` transaction
- **Load balancing**: Multiple Electrum servers with parallel broadcasting
- **Configuration changes**: `electrum_rpc_url` → `electrum_rpc_urls` (array)

## Technical Architecture Changes

### 1. **Monero Integration Overhaul**
The shift from RPC to direct FFI bindings represents a fundamental architectural change:
- **Performance**: Direct function calls are faster than RPC communication
- **Reliability**: Eliminates RPC communication failures
- **Security**: Reduced attack surface by removing RPC interface
- **Complexity**: Direct integration of Monero codebase

### 2. **Enhanced Swap Protocol**
- **Early refund mechanism**: Reduces wait times for failed swaps
- **Improved timeout handling**: Better edge case management
- **Atomic guarantees**: Maintained while improving efficiency

### 3. **Network Resilience**
- **Multi-server architecture**: Reduced single points of failure
- **Peer caching**: Improved connection reliability
- **Load balancing**: Better distribution of network requests

## Development Activity

### Recent Development Stats
- **Latest commit**: 7042b1b0 (release: 2.2.0-beta)
- **Recent activity**: 10+ commits since December 2024
- **Active development**: Consistent bug fixes and feature additions

### Key Contributors
- Active bot-assisted releases (@unstoppableswap-botty)
- Community contributions and issue reports
- Coordinated release management

## Breaking Changes and Migration

### For ASB Operators (Docker Users)
```bash
# Migration commands for wallet files:

# Testnet
cp /var/lib/docker/volumes/testnet_stagenet_monero-wallet-rpc-data/_data/* \
   /var/lib/docker/volumes/testnet_testnet_asb-data/_data/monero/wallets

# Mainnet  
cp /var/lib/docker/volumes/mainnet_mainnet_monero-wallet-rpc-data/_data/* \
   /var/lib/docker/volumes/mainnet_mainnet_asb-data/_data/monero/wallets
```

### For Script Users
Log piping changes required:
```bash
# Before
asb logs | my-script.sh
asb logs > output.txt

# After  
asb logs 2>&1 | my-script.sh
asb logs > output.txt 2>&1
```

## Future Outlook

The UnstoppableSwap project continues active development with focus on:
- **Performance improvements** through direct Monero integration
- **Network reliability** through better peer discovery and load balancing  
- **User experience** improvements with better error handling and retry logic
- **Cross-platform support** with Flatpak and other distribution methods

The project maintains its commitment to decentralized, trustless atomic swaps while significantly improving the technical implementation and user experience.

## Repository Information

- **GitHub**: https://github.com/UnstoppableSwap/core
- **Latest Release**: 2.2.0-beta (June 17, 2025)
- **Fork of**: comit-network/xmr-btc-swap  
- **Stars**: 107
- **License**: Open source
- **Primary Language**: Rust

The project represents one of the most mature implementations of Bitcoin-Monero atomic swaps, with continuous development focused on reliability, performance, and user experience improvements.
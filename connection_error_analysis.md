# Connection Error Handling and Progress Updates Analysis

## Overview

The error message "Trying to reconnect (Last Error: Connection Timeout, 12 times failed, retries_left: 20)" suggests a connection retry mechanism that tracks failed attempts and remaining retries. While this exact format isn't found in the main codebase, several connection handling mechanisms exist.

## Key Connection Handling Components

### 1. Redial Behavior (`swap/src/network/redial.rs`)

The redial behavior implements automatic reconnection with exponential backoff:

```rust
pub struct Behaviour {
    peer: PeerId,
    sleep: Option<Pin<Box<Sleep>>>,
    backoff: ExponentialBackoff,
}
```

**Key Features:**
- Exponential backoff for reconnection attempts
- Tracks time until next redial attempt
- Never gives up reconnecting (`max_elapsed_time: None`)
- Logs: `"Waiting for next redial attempt"`

### 2. Event Loop Connection Handling (`swap/src/cli/event_loop.rs`)

**Connection Events Logged:**
- `"Connected to Alice"` - successful connection
- `"Dialing Alice"` - connection attempt
- `"Lost connection to Alice"` - connection failure
- `"Failed to connect to Alice"` - outgoing connection error

**Retry Mechanisms:**
- Uses `backoff::future::retry_notify` for various operations
- Different timeout configurations:
  - `REQUEST_RESPONSE_PROTOCOL_TIMEOUT: 60s`
  - `EXECUTION_SETUP_PROTOCOL_TIMEOUT: 120s`

### 3. GUI Progress Updates (`src-gui/`)

The frontend tracks various connection states and progress:

#### Registry Connection Tracking (`src-gui/src/store/features/makersSlice.ts`)
```typescript
registry: {
  makers: ExtendedMakerStatus[] | null;
  connectionFailsCount: number; // Tracks failed connection attempts
}
```

#### Background Progress Events (`src-gui/src/renderer/background.ts`)
- Listens for various progress events including connection status
- Updates UI based on connection state changes

#### Tor Connection Status (`src-gui/src/store/features/torSlice.ts`)
- Tracks Tor proxy status and bootstrap progress
- Handles connection state for anonymized connections

### 4. Monero Wallet Connection Retry (`swap/src/monero/wallet.rs`)

```rust
for i in 1..=max_attempts {
    tracing::info!(name = %self.main_wallet, attempt=i, "Syncing Monero wallet");
    
    match result {
        Ok(refreshed) => return Ok(refreshed),
        Err(error) => {
            let attempts_left = max_attempts - i;
            tracing::warn!(
                attempt=i, 
                %attempts_left, 
                %error, 
                "Failed to sync Monero wallet"
            );
        }
    }
}
```

## Potential Sources of the Specific Error Message

Given the error format "Trying to reconnect (Last Error: Connection Timeout, 12 times failed, retries_left: 20)", this could originate from:

### 1. Monero Wallet RPC Process
The application downloads and runs external Monero wallet RPC binaries that might generate this message format.

### 2. GUI Translation Layer
The frontend might be formatting backend error messages into user-friendly text.

### 3. External Dependencies
Libraries like libp2p or networking dependencies might generate these messages.

## Recommendations for Improvement

### 1. Enhanced Progress Reporting

**Current State:** Basic logging of retry attempts
**Improvement:** Implement structured progress updates with:
- Current attempt number
- Total attempts made
- Remaining retries (if applicable)
- Last error details
- Estimated time until next retry

### 2. Unified Error Formatting

**Implementation suggestion:**
```rust
#[derive(Debug, Clone)]
pub struct ConnectionProgress {
    pub current_attempt: u32,
    pub total_attempts: u32,
    pub retries_left: Option<u32>,
    pub last_error: String,
    pub next_retry_in: Option<Duration>,
}
```

### 3. GUI Progress Indicators

**Current Features:**
- Linear progress bars for known progress
- Circular progress for unknown progress
- Connection status alerts

**Improvements:**
- Show detailed retry information
- Display connection quality metrics
- Provide user control over retry behavior

### 4. Better Error Context

**Current Issues:**
- Generic error messages
- Limited context about connection state
- No user guidance on resolution

**Solutions:**
- Categorize connection errors by type
- Provide actionable suggestions
- Show network diagnostics

## Connection Flow Analysis

### P2P Connection (Alice ↔ Bob)
1. Initial dial attempt
2. Connection establishment or failure
3. Redial behavior kicks in on failure
4. Exponential backoff with unlimited retries
5. Progress updates via GUI events

### Monero Node Connection
1. Daemon selection from available nodes
2. Health check with 30s timeout
3. Fallback to next available daemon
4. Wallet RPC process management

### Bitcoin Node Connection
1. Electrum server selection
2. Connection status monitoring
3. Automatic failover between nodes
4. Balance and transaction monitoring

## Diagnostic Information

To better understand the specific error message, check:

1. **Monero wallet RPC logs** - Look for connection retry messages
2. **Tor connection logs** - Check for circuit establishment issues  
3. **GUI console logs** - Look for formatted error messages
4. **System network logs** - Check for underlying network issues

## Configuration Options

### Timeout Settings
- `bitcoin_lock_mempool_timeout`: 10 minutes (mainnet)
- `bitcoin_lock_confirmed_timeout`: 2 hours (mainnet) 
- Monero daemon request timeout: 30 seconds
- Request-response protocol timeout: 60 seconds

### Retry Behavior
- Redial: Unlimited retries with exponential backoff
- Swap setup: Limited by `EXECUTION_SETUP_PROTOCOL_TIMEOUT`
- Encrypted signature: Unlimited retries
- Registry updates: Tracked failure count with threshold

This analysis provides a foundation for understanding and improving the connection error handling and progress update mechanisms in the swap application.
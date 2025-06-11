# Test Failure Analysis: `alice_and_bob_refund_using_cancel_and_refund_command_timelock_not_expired`

## Issue Summary

The test `given_alice_and_bob_manually_cancel_when_timelock_not_expired_errors` is failing because the `parse_rpc_error_code` function cannot extract the RPC error code from the error returned by the cancel operation.

## Root Cause

1. **The expected behavior**: When trying to cancel a swap before the timelock expires, the Bitcoin network should reject the transaction with error code `-26` ("non-BIP68-final") because the transaction uses BIP68 sequence numbers (relative timelocks) that haven't been satisfied yet.

2. **The actual error flow**:
   - The `submit_tx_cancel` function calls `bitcoin_wallet.broadcast(transaction, "cancel")`
   - The `broadcast` function in `swap/src/bitcoin/wallet.rs` calls the electrum client's `transaction_broadcast` method
   - When the electrum server rejects the transaction, it returns an error like: `"sendrawtransaction RPC error: {\"code\":-26,\"message\":\"non-BIP68-final\"}"`
   - However, the `broadcast` function wraps this error with additional context using `.with_context()`:
     ```rust
     client
         .transaction_broadcast(&transaction)
         .with_context(|| {
             format!("Failed to broadcast Bitcoin {} transaction {}", kind, txid)
         })?;
     ```

3. **The problem with `parse_rpc_error_code`**:
   - The function expects to downcast the error to `bdk_electrum::electrum_client::Error::Protocol`
   - But the error is now wrapped in an anyhow context error, so the downcast fails
   - This causes the error: `"Error is of incorrect variant. We expected an Electrum error, but got: Bitcoin cancel transaction..."`

## Current Error Chain

```
anyhow::Error (from .with_context())
└── bdk_electrum::electrum_client::Error::Protocol
    └── serde_json::Value::String("sendrawtransaction RPC error: {\"code\":-26,\"message\":\"non-BIP68-final\"}")
```

The `parse_rpc_error_code` function tries to downcast directly to `bdk_electrum::electrum_client::Error` but fails because the error is wrapped.

## Solution

The `parse_rpc_error_code` function needs to be updated to handle wrapped errors. It should traverse the error chain to find the underlying electrum error. Here's the fix:

```rust
pub fn parse_rpc_error_code(error: &anyhow::Error) -> anyhow::Result<i64> {
    // First try to downcast directly
    let string = match error.downcast_ref::<bdk_electrum::electrum_client::Error>() {
        Some(bdk_electrum::electrum_client::Error::Protocol(serde_json::Value::String(string))) => {
            string
        }
        _ => {
            // If direct downcast fails, try to find the error in the chain
            for cause in error.chain() {
                if let Some(bdk_electrum::electrum_client::Error::Protocol(serde_json::Value::String(string))) 
                    = cause.downcast_ref::<bdk_electrum::electrum_client::Error>() {
                    return parse_rpc_error_string(string);
                }
            }
            
            bail!(
                "Error is of incorrect variant. We expected an Electrum error, but got: {}",
                error
            )
        }
    };

    parse_rpc_error_string(string)
}

fn parse_rpc_error_string(string: &str) -> anyhow::Result<i64> {
    let json = serde_json::from_str(&string.replace("sendrawtransaction RPC error:", ""))?;

    let json_map = match json {
        serde_json::Value::Object(map) => map,
        _ => bail!("Json error is not json object "),
    };

    let error_code_value = match json_map.get("code") {
        Some(val) => val,
        None => bail!("No error code field"),
    };

    let error_code_number = match error_code_value {
        serde_json::Value::Number(num) => num,
        _ => bail!("Error code is not a number"),
    };

    if let Some(int) = error_code_number.as_i64() {
        Ok(int)
    } else {
        bail!("Error code is not an unsigned integer")
    }
}
```

## Test Verification

After implementing this fix, the test should:
1. Successfully broadcast the cancel transaction but have it rejected by the Bitcoin network with "non-BIP68-final"
2. The `parse_rpc_error_code` function should successfully extract the `-26` error code
3. The test assertion should pass: `assert_eq!(parse_rpc_error_code(&error).unwrap(), i64::from(RpcErrorCode::RpcVerifyRejected))`

## Files to Modify

- `swap/src/bitcoin.rs` - Update the `parse_rpc_error_code` function to handle wrapped errors
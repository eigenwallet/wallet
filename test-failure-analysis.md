# Test Failure Analysis: `alice_and_bob_refund_using_cancel_and_refund_command_timelock_not_expired`

## Issue Summary

The test `given_alice_and_bob_manually_cancel_when_timelock_not_expired_errors` is failing because the `parse_rpc_error_code` function cannot extract the RPC error code from a **multi-error** returned by the cancel operation.

## Root Cause

1. **The expected behavior**: When trying to cancel a swap before the timelock expires, the Bitcoin network should reject the transaction with error code `-26` ("non-BIP68-final") because the transaction uses BIP68 sequence numbers (relative timelocks) that haven't been satisfied yet.

2. **The actual error structure**: The broadcast operation returns a **multi-error** that aggregates errors from multiple electrum servers:
   ```
   Bitcoin cancel transaction 1fa267e681bcc1175c2fa90119490e6e96b6bc9ecba329eb652cd4321c409728 failed to broadcast on all 1 servers: 1 errors occurred
     1: Electrum server error: "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"non-BIP68-final\"}"
   ```

3. **The problem**: The test is trying to call `parse_rpc_error_code` directly on the multi-error, but this function expects a single electrum error, not an aggregated multi-error.

## Current Error Chain

```
Multi-Error (electrum balancer)
├── Error 1: bdk_electrum::electrum_client::Error::Protocol
│   └── serde_json::Value::String("sendrawtransaction RPC error: {\"code\":-26,\"message\":\"non-BIP68-final\"}")
├── Error 2: ...
└── Error N: ...
```

The `parse_rpc_error_code` function tries to downcast directly to `bdk_electrum::electrum_client::Error` but the multi-error has a different type structure.

## Solution

Instead of trying to parse the RPC error code from the top-level multi-error, we should use the `.any(...)` method on the multi-error to check if **any** of the underlying errors matches the expected RPC error code.

The test assertion should be updated from:
```rust
assert_eq!(
    parse_rpc_error_code(&error).unwrap(),
    i64::from(RpcErrorCode::RpcVerifyRejected)
);
```

To something like:
```rust
assert!(error.any(|err| {
    parse_rpc_error_code(err)
        .map(|code| code == i64::from(RpcErrorCode::RpcVerifyRejected))
        .unwrap_or(false)
}));
```

Or alternatively, we could extend the `parse_rpc_error_code` function to handle multi-errors by iterating through all the underlying errors and checking if any of them contains the expected RPC error code.

## Test Verification

After implementing this fix, the test should:
1. Successfully broadcast the cancel transaction but have it rejected by the Bitcoin network with "non-BIP68-final"
2. The multi-error should be properly handled to extract the `-26` error code from one of its underlying errors
3. The test assertion should pass by checking if any of the errors in the multi-error matches `RpcErrorCode::RpcVerifyRejected`

## Files to Modify

- The failing test file: `swap/tests/alice_and_bob_refund_using_cancel_and_refund_command_timelock_not_expired.rs` - Update the assertion to use `.any(...)` on the multi-error
- Optionally: `swap/src/bitcoin.rs` - Extend `parse_rpc_error_code` to handle multi-errors
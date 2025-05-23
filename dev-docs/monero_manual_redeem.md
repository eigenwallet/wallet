# Manual Monero Redeem (Bob)

This short guide covers how to recover the Monero funds for a swap wallet without scanning the entire chain.
It applies when the GUI acts as **Bob** and you need to redeem the locked Monero.

1. Query the current restore height from your connected `monero-wallet-rpc` using `get_height`.
2. When opening the swap wallet, set this value as the restore height.
3. Call `scanTransactions` with the `xmr_lock` transaction id to manually add the lock transaction to the wallet.
   This avoids rescanning from the genesis block.

After scanning the single transaction the wallet can sweep the funds normally.

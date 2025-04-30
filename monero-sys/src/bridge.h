#pragma once

#include <memory>

#include "../monero/src/wallet/api/wallet2_api.h"

namespace Monero
{
    /**
     * CXX doesn't support static methods as yet, so we define free functions here that simply
     * call the appropriate static methods.
     */
    inline WalletManager *getWalletManager()
    {
        // This causes the wallet to print some logging to stdout
        // This is useful for debugging
        // TODO: Only enable in debug releases or expose setLogLevel as a FFI function
        WalletManagerFactory::setLogLevel(2);

        return WalletManagerFactory::getWalletManager();
    }

    /**
     * CXX also doesn't support returning strings by value from C++ to Rust, so we wrap those
     * in a unique_ptr.
     */
    inline std::unique_ptr<std::string> address(const Wallet &wallet, uint32_t account_index, uint32_t address_index)
    {
        auto addr = wallet.address(account_index, address_index);
        return std::make_unique<std::string>(addr);
    }

    /**
     * Same as for [`address`]
     */
    inline std::unique_ptr<std::string> walletManagerErrorString(WalletManager &manager)
    {
        auto err = manager.errorString();
        return std::make_unique<std::string>(err);
    }

    /**
     * Wrapper for Wallet::checkTxKey to accommodate passing std::string by reference.
     * The original API takes the tx_key parameter by value which is not compatible
     * with cxx. Taking it by const reference here allows us to expose the function
     * to Rust safely while still calling the original method internally.
     */
    inline bool checkTxKey(
        Wallet &wallet,
        const std::string &txid,
        const std::string &tx_key,
        const std::string &address,
        uint64_t &received,
        bool &in_pool,
        uint64_t &confirmations)
    {
        return wallet.checkTxKey(txid, tx_key, address, received, in_pool, confirmations);
    }

    /**
     * A wrapper around Wallet::createTransaction which passes sensible defaults and doesn't
     * require an optional argument which CXX doesn't support.
     */
    inline PendingTransaction *createTransaction(
        Wallet &wallet,
        const std::string &dest_address,
        u_int64_t amount)
    {
        return wallet.createTransaction(dest_address, "", Monero::optional<uint64_t>(), 1, PendingTransaction::Priority_Default);
    }
}
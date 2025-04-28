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
}
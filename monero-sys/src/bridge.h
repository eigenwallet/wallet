#pragma once

#include <memory>

#include "../monero/src/wallet/api/wallet2_api.h"

namespace Monero
{
    using ConnectionStatus = Wallet::ConnectionStatus;

    /**
     * CXX doesn't support static methods as yet, so we define free functions here that simply
     * call the appropriate static methods.
     */
    inline WalletManager *getWalletManager()
    {
        // This causes the wallet to print some logging to stdout
        // This is useful for debugging
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
     * Get the error string of a pending transaction.
     */
    inline std::unique_ptr<std::string> pendingTransactionErrorString(const PendingTransaction &tx)
    {
        auto err = tx.errorString();
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
     * Get the path of the wallet.
     */
    inline std::unique_ptr<std::string> walletPath(const Wallet &wallet)
    {
        return std::make_unique<std::string>(wallet.path());
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
        return wallet.createTransaction(dest_address, "", Monero::optional<uint64_t>(amount), 0, PendingTransaction::Priority_Default);
    }

    inline PendingTransaction *createSweepTransaction(
        Wallet &wallet,
        const std::string &dest_address)
    {
        return wallet.createTransaction(dest_address, "", Monero::optional<uint64_t>(), 0, PendingTransaction::Priority_Default);
    }

    inline bool setWalletDaemon(Wallet &wallet, const std::string &daemon_address)
    {
        return wallet.setDaemon(daemon_address);
    }

    inline std::unique_ptr<std::string> pendingTransactionTxId(const PendingTransaction &tx)
    {
        const auto ids = tx.txid();
        if (ids.empty())
            return std::make_unique<std::string>("");
        return std::make_unique<std::string>(ids.front());
    }

    /**
     * Get the transaction key for a given transaction id
     */
    inline std::unique_ptr<std::string> walletGetTxKey(const Wallet &wallet, const std::string &txid)
    {
        auto key = wallet.getTxKey(txid);
        return std::make_unique<std::string>(key);
    }

    inline std::unique_ptr<std::vector<std::string>> pendingTransactionTxIds(const PendingTransaction &tx)
    {
        return std::make_unique<std::vector<std::string>>(tx.txid());
    }
}

#include "easylogging++.h"
#include "bridge.h"
#include "monero-sys/src/bridge.rs.h"

namespace monero_rust_log
{
    // One dispatch callback instance for the whole program.
    class RustDispatch final : public el::LogDispatchCallback
    {
    protected:
        void handle(const el::LogDispatchData *data) noexcept override
        {
            auto *m = data->logMessage();

            uint8_t level;
            switch (m->level())
            {
            case el::Level::Trace:
                level = 0;
                break;
            case el::Level::Debug:
                level = 1;
                break;
            case el::Level::Info:
                level = 2;
                break;
            case el::Level::Warning:
                level = 3;
                break;
            case el::Level::Error:
            case el::Level::Fatal:
                level = 4;
                break;
            default:
                level = 2; // Default to info.
                break;
            }

            // Forward to Rust.
            monero_rust_log::forward_cpp_log(
                level,
                m->file().length() > 0 ? m->file() : "",
                m->line(),
                m->func(),
                m->message());
        }
    };

    bool installed = false;

    inline void install_log_callback()
    {
        if (installed)
            return;
        installed = true;

        // Make sure easylogging++ itself is initialised (usually already done
        // because the Monero libs call el::Helpers::setThreadName etc.).
        el::Helpers::installLogDispatchCallback<RustDispatch>("rust-forward");

        // Disable all default easylogging++ log writers such that messages are **only**
        // forwarded through the RustDispatch callback above. This prevents them from
        // being printed directly to stdout/stderr or written to files.
        el::Loggers::reconfigureAllLoggers(el::ConfigurationType::ToStandardOutput, "false");
        el::Loggers::reconfigureAllLoggers(el::ConfigurationType::ToFile, "false");

        // Make the above configuration the *default* for any loggers that may be
        // created after this point (many Monero components lazily create their
        // own logger instances). Without this, newly-created loggers would revert
        // to printing to stdout again.
        el::Configurations defaultConf;
        defaultConf.set(el::Level::Global, el::ConfigurationType::ToStandardOutput, "false");
        defaultConf.set(el::Level::Global, el::ConfigurationType::ToFile, "false");
        el::Loggers::setDefaultConfigurations(defaultConf, true /* enable default for new loggers */);
    }
} // namespace

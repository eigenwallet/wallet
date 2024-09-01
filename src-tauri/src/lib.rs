use std::result::Result;
use std::sync::Arc;
use swap::cli::{
    api::{
        request::{
            BalanceArgs, BuyXmrArgs, GetHistoryArgs, GetSwapInfosAllArgs, ResumeSwapArgs,
            SuspendCurrentSwapArgs, WithdrawBtcArgs,
        },
        tauri_bindings::{TauriContextStatusEvent, TauriEmitter, TauriHandle},
        Context, ContextBuilder,
    },
    command::{Bitcoin, Monero},
};
use tauri::{async_runtime::RwLock, Manager, RunEvent};

trait ToStringResult<T> {
    fn to_string_result(self) -> Result<T, String>;
}

// Implement the trait for Result<T, E>
impl<T, E: ToString> ToStringResult<T> for Result<T, E> {
    fn to_string_result(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

/// This macro is used to create boilerplate functions as tauri commands
/// that simply delegate handling to the respective request type.
///
/// # Example
/// ```ignored
/// tauri_command!(get_balance, BalanceArgs);
/// ```
/// will resolve to
/// ```ignored
/// #[tauri::command]
/// async fn get_balance(context: tauri::State<'...>, args: BalanceArgs) -> Result<BalanceArgs::Response, String> {
///     args.handle(context.inner().clone()).await.to_string_result()
/// }
///
/// # Example 2
/// ```ignored
/// tauri_command!(get_balance, BalanceArgs, no_args);
/// ```
/// will resolve to
/// ```ignored
/// #[tauri::command]
/// async fn get_balance(context: tauri::State<'...>) -> Result<BalanceArgs::Response, String> {
///    BalanceArgs {}.handle(context.inner().clone()).await.to_string_result()
/// }
/// ```
macro_rules! tauri_command {
    ($fn_name:ident, $request_name:ident) => {
        #[tauri::command]
        async fn $fn_name(
            context: tauri::State<'_, RwLock<State>>,
            args: $request_name,
        ) -> Result<<$request_name as swap::cli::api::request::Request>::Response, String> {
            // Throw error if context is not available
            let context = context
                .read()
                .await
                .context
                .clone()
                .ok_or("Context not available")?;

            <$request_name as swap::cli::api::request::Request>::request(args, context)
                .await
                .to_string_result()
        }
    };
    ($fn_name:ident, $request_name:ident, no_args) => {
        #[tauri::command]
        async fn $fn_name(
            context: tauri::State<'_, RwLock<State>>,
        ) -> Result<<$request_name as swap::cli::api::request::Request>::Response, String> {
            // Throw error if context is not available
            let context = context
                .read()
                .await
                .context
                .clone()
                .ok_or("Context not available")?;

            <$request_name as swap::cli::api::request::Request>::request($request_name {}, context)
                .await
                .to_string_result()
        }
    };
}

struct State {
    pub context: Option<Arc<Context>>,
}

impl State {
    fn new() -> Self {
        Self { context: None }
    }

    fn set_context(&mut self, context: impl Into<Option<Arc<Context>>>) {
        self.context = context.into();
    }
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.app_handle().to_owned();

    app_handle.manage::<RwLock<State>>(RwLock::new(State::new()));

    tauri::async_runtime::spawn(async move {
        let tauri_handle = TauriHandle::new(app_handle.clone());

        let context = ContextBuilder::new(true)
            .with_bitcoin(Bitcoin {
                bitcoin_electrum_rpc_url: None,
                bitcoin_target_block: None,
            })
            .with_monero(Monero {
                monero_daemon_address: None,
            })
            .with_json(false)
            .with_debug(true)
            .with_tauri(tauri_handle.clone())
            .build()
            .await;

        match context {
            Ok(context) => {
                let state = app_handle.state::<RwLock<State>>();

                state.write().await.set_context(Arc::new(context));

                tauri_handle.emit_context_init_progress_event(TauriContextStatusEvent::Available);
            }
            Err(e) => {
                println!("Error while initializing context: {:?}", e);

                tauri_handle.emit_context_init_progress_event(TauriContextStatusEvent::Failed);
            }
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_balance,
            get_swap_infos_all,
            withdraw_btc,
            buy_xmr,
            resume_swap,
            get_history,
            suspend_current_swap
        ])
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            RunEvent::Exit | RunEvent::ExitRequested { .. } => {
                let context = app.state::<RwLock<State>>().inner().try_read();

                match context {
                    Ok(context) => {
                        if let Some(context) = context.context.as_ref() {
                            if let Err(err) = context.cleanup() {
                                println!("Cleanup failed {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Failed to acquire lock on context: {}", err);
                    }
                }
            }
            _ => {}
        })
}

tauri_command!(get_balance, BalanceArgs);
tauri_command!(buy_xmr, BuyXmrArgs);
tauri_command!(resume_swap, ResumeSwapArgs);
tauri_command!(withdraw_btc, WithdrawBtcArgs);
tauri_command!(suspend_current_swap, SuspendCurrentSwapArgs, no_args);
tauri_command!(get_swap_infos_all, GetSwapInfosAllArgs, no_args);
tauri_command!(get_history, GetHistoryArgs, no_args);

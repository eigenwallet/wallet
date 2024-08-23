use std::result::Result;
use std::sync::Arc;
use swap::{
    api::{
        request::{
            buy_xmr as buy_xmr_impl, get_balance as get_balance_impl,
            get_history as get_history_impl, get_swap_infos_all as get_swap_infos_all_impl,
            resume_swap as resume_swap_impl, suspend_current_swap as suspend_current_swap_impl,
            withdraw_btc as withdraw_btc_impl, BalanceArgs, BalanceResponse, BuyXmrArgs,
            BuyXmrResponse, GetHistoryResponse, GetSwapInfoResponse, ResumeArgs,
            ResumeSwapResponse, SuspendCurrentSwapResponse, WithdrawBtcArgs, WithdrawBtcResponse,
        },
        Context,
    },
    cli::command::{Bitcoin, Monero},
};
use tauri::{Manager, RunEvent, State};

trait ToStringResult<T> {
    fn to_string_result(self) -> Result<T, String>;
}

// Implement the trait for Result<T, E>
impl<T, E: ToString> ToStringResult<T> for Result<T, E> {
    fn to_string_result(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_balance(
    context: State<'_, Arc<Context>>,
    args: BalanceArgs,
) -> Result<BalanceResponse, String> {
    get_balance_impl(args, context.inner().clone())
        .await
        .to_string_result()
}

#[tauri::command]
async fn get_swap_infos_all(
    context: State<'_, Arc<Context>>,
) -> Result<Vec<GetSwapInfoResponse>, String> {
    get_swap_infos_all_impl(context.inner().clone())
        .await
        .to_string_result()
}

#[tauri::command]
async fn buy_xmr(
    context: State<'_, Arc<Context>>,
    args: BuyXmrArgs,
) -> Result<BuyXmrResponse, String> {
    buy_xmr_impl(args, context.inner().clone())
        .await
        .to_string_result()
}

#[tauri::command]
async fn get_history(context: State<'_, Arc<Context>>) -> Result<GetHistoryResponse, String> {
    get_history_impl(context.inner().clone())
        .await
        .to_string_result()
}

#[tauri::command]
async fn resume_swap(
    context: State<'_, Arc<Context>>,
    args: ResumeArgs,
) -> Result<ResumeSwapResponse, String> {
    resume_swap_impl(args, context.inner().clone())
        .await
        .to_string_result()
}

#[tauri::command]
async fn withdraw_btc(
    context: State<'_, Arc<Context>>,
    args: WithdrawBtcArgs,
) -> Result<WithdrawBtcResponse, String> {
    withdraw_btc_impl(args, context.inner().clone())
        .await
        .to_string_result()
}

#[tauri::command]
async fn suspend_current_swap(
    context: State<'_, Arc<Context>>,
) -> Result<SuspendCurrentSwapResponse, String> {
    suspend_current_swap_impl(context.inner().clone())
        .await
        .to_string_result()
}

fn setup<'a>(app: &'a mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    tauri::async_runtime::block_on(async {
        let context = Context::build(
            Some(Bitcoin {
                bitcoin_electrum_rpc_url: None,
                bitcoin_target_block: None,
            }),
            Some(Monero {
                monero_daemon_address: None,
            }),
            None,
            None,
            true,
            true,
            true,
            None,
        )
        .await
        .unwrap()
        .with_tauri_handle(app.app_handle().to_owned());

        app.manage(Arc::new(context));
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
                let context = app.state::<Arc<Context>>().inner();

                if let Err(err) = context.cleanup() {
                    println!("Cleanup failed {}", err);
                }
            }
            _ => {}
        })
}

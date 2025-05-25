import { getLogsOfSwap, saveLogFiles } from 'renderer/rpc'
import PromiseInvokeButton from 'renderer/components/PromiseInvokeButton'
import { store } from 'renderer/store/storeRenderer'

export default function ExportLogsButton(props: { swap_id: string }) {
    async function exportLogs() {
        const swap_logs = await getLogsOfSwap(props.swap_id, false)
        const daemon_logs = store.getState().rpc?.logs

        await saveLogFiles({
            swap_logs: swap_logs.logs.join('\n'),
            daemon_logs: daemon_logs?.join('\n'),
        })
    }

    return (
        <PromiseInvokeButton onInvoke={() => exportLogs()} {...props}>
            Export Logs
        </PromiseInvokeButton>
    )
}

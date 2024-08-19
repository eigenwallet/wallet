import { Tooltip } from "@material-ui/core";
import { ButtonProps } from "@material-ui/core/Button/Button";
import { green, red } from "@material-ui/core/colors";
import DoneIcon from "@material-ui/icons/Done";
import ErrorIcon from "@material-ui/icons/Error";
import PlayArrowIcon from "@material-ui/icons/PlayArrow";
import { GetSwapInfoArgs, GetSwapInfoResponse } from "models/tauriModel";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import { resumeSwap } from "renderer/rpc";
import { SwapStateName } from "../../../../../models/rpcModel";

export function SwapResumeButton({
  swap,
  ...props
}: ButtonProps & { swap: GetSwapInfoResponse }) {
  return (
    <PromiseInvokeButton
      variant="contained"
      color="primary"
      disabled={swap.completed}
      endIcon={<PlayArrowIcon />}
      onClick={() => resumeSwap(swap.swap_id)}
      {...props}
    >
      Resume
    </PromiseInvokeButton>
  );
}

export function SwapCancelRefundButton({
  swap,
  ...props
}: { swap: GetSwapInfoArgs } & ButtonProps) {
  /*
  TODO: Reimplement this using Tauri
  */
  return <> </>;

  const cancelOrRefundable =
    isSwapStateNamePossiblyCancellableSwap(swap.state_name) ||
    isSwapStateNamePossiblyRefundableSwap(swap.state_name);

  if (!cancelOrRefundable) {
    return <></>;
  }

  return (
    <PromiseInvokeButton
      displayErrorSnackbar={false}
      {...props}
      onClick={async () => {
        throw new Error("Not implemented");
      }}
    >
      Attempt manual Cancel & Refund
    </PromiseInvokeButton>
  );
}

export default function HistoryRowActions(swap: GetSwapInfoResponse) {
  // TODO: Fix this to use the new state names (SwapStateName is not equivalent to Display impl of BobState)
  if (swap.state_name === SwapStateName.XmrRedeemed) {
    return (
      <Tooltip title="The swap is completed because you have redeemed the XMR">
        <DoneIcon style={{ color: green[500] }} />
      </Tooltip>
    );
  }

  // TODO: Fix this to use the new state names (SwapStateName is not equivalent to Display impl of BobState)
  if (swap.state_name === SwapStateName.BtcRefunded) {
    return (
      <Tooltip title="The swap is completed because your BTC have been refunded">
        <DoneIcon style={{ color: green[500] }} />
      </Tooltip>
    );
  }

  // TODO: Fix this to use the new state names (SwapStateName is not equivalent to Display impl of BobState)
  if (swap.state_name === SwapStateName.BtcPunished) {
    return (
      <Tooltip title="The swap is completed because you have been punished">
        <ErrorIcon style={{ color: red[500] }} />
      </Tooltip>
    );
  }

  return <SwapResumeButton swap={swap} />;
}

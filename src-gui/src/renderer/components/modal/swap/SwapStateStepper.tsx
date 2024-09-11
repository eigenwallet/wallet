import { Step, StepLabel, Stepper, Typography } from "@material-ui/core";
import { SwapSpawnType } from "models/cliModel";
import { SwapSlice } from "models/storeModel";
import { useAppSelector } from "store/hooks";
import { exhaustiveGuard } from "utils/typescriptUtils";

export enum PathType {
  HAPPY_PATH = "happy path",
  UNHAPPY_PATH = "unhappy path",
}

/**
 * Determines the current step in the swap process based on the previous and latest state.
 * @param prevState - The previous state of the swap process (null if it's the initial state)
 * @param latestState - The latest state of the swap process
 * @returns A tuple containing [PathType, activeStep, errorFlag]
 */
function getActiveStep(
  state: SwapSlice["state"],
): [type: PathType, step: number, isError: boolean] {
  if (state === null) {
    return [PathType.HAPPY_PATH, 0, false];
  }

  const prevState = state.prev;
  let latestState = state.curr;

  const processExited = latestState.type === "Released";

  // If the swap is completed we use the previous state to display the correct step
  if (latestState.type === "Released") {
    latestState = prevState;
  }

  switch (latestState.type) {
    // Step 0: Initializing the swap
    // These states represent the very beginning of the swap process
    // No funds have been locked
    case "Initiated":
    case "ReceivedQuote":
    case "WaitingForBtcDeposit":
    case "Started":
      return [PathType.HAPPY_PATH, 0, processExited];

    // Step 1: Waiting for Bitcoin lock confirmation
    // Bitcoin has been locked, waiting for the counterparty to lock their XMR
    case "BtcLockTxInMempool":
      return [PathType.HAPPY_PATH, 1, processExited];

    // Still Step 1: Both Bitcoin and XMR have been locked, waiting for Monero lock to be confirmed
    case "XmrLockTxInMempool":
      return [PathType.HAPPY_PATH, 1, processExited];

    // Step 2: Waiting for encrypted signature to be sent to Alice
    // and for Alice to redeem the Bitcoin
    case "XmrLocked":
      // case BobStateName.EncSigSent: // TODO: There is no equivalent for this with Tauri events
      return [PathType.HAPPY_PATH, 2, processExited];

    // Step 3: Waiting for XMR redemption
    // Bitcoin has been redeemed by Alice, now waiting for us to redeem Monero
    case "BtcRedeemed":
      return [PathType.HAPPY_PATH, 3, processExited];

    // Step 4: Swap completed successfully
    // XMR redemption transaction is in mempool, swap is essentially complete
    case "XmrRedeemInMempool":
      return [PathType.HAPPY_PATH, 4, false];

    // Edge Case of Happy Path where the swap is safely aborted. We "fail" at the first step.
    // TODO: There's no equivalent for this with the Tauri events
    // case BobStateName.SafelyAborted:
    //  return [PathType.HAPPY_PATH, 0, true];

    // // Unhappy Path
    // Step: 1 (Cancelling swap, checking if cancel transaction has been published already by the other party)
    // TODO: There's no equivalent for this with the Tauri events
    //case BobStateName.CancelTimelockExpired:
    //  return [PathType.UNHAPPY_PATH, 0, processExited];

    // Unhappy Path States

    // Step 2: Swap has been cancelled. Waiting for Bitcoin to be refunded
    case "BtcCancelled":
      return [PathType.UNHAPPY_PATH, 1, processExited];

    // Step 2: Swap cancelled and Bitcoin refunded successfully
    case "BtcRefunded":
      return [PathType.UNHAPPY_PATH, 2, false];

    // Step 2 (Failed): Failed to refund Bitcoin
    // The timelock expired before we could refund, resulting in punishment
    case "BtcPunished":
      return [PathType.UNHAPPY_PATH, 1, true];

    // Attempting cooperative redemption after punishment
    case "AttemptingCooperativeRedeem":
    case "CooperativeRedeemAccepted":
      return [PathType.UNHAPPY_PATH, 1, false];
    case "CooperativeRedeemRejected":
      return [PathType.UNHAPPY_PATH, 1, true];

    case "Released":
      throw new Error(
        "Unexpected, latest and previous state cannot be 'Released'",
      );

    default:
      return exhaustiveGuard(stateName);
  }
}

function HappyPathStepper({
  activeStep,
  error,
}: {
  activeStep: number;
  error: boolean;
}) {
  return (
    <Stepper activeStep={activeStep}>
      <Step key={0}>
        <StepLabel
          optional={<Typography variant="caption">~12min</Typography>}
          error={error && activeStep === 0}
        >
          Locking your BTC
        </StepLabel>
      </Step>
      <Step key={1}>
        <StepLabel
          optional={<Typography variant="caption">~18min</Typography>}
          error={error && activeStep === 1}
        >
          They lock their XMR
        </StepLabel>
      </Step>
      <Step key={2}>
        <StepLabel
          optional={<Typography variant="caption">~2min</Typography>}
          error={error && activeStep === 2}
        >
          They redeem the BTC
        </StepLabel>
      </Step>
      <Step key={3}>
        <StepLabel
          optional={<Typography variant="caption">~2min</Typography>}
          error={error && activeStep === 3}
        >
          Redeeming your XMR
        </StepLabel>
      </Step>
    </Stepper>
  );
}

function UnhappyPathStepper({
  activeStep,
  error,
}: {
  activeStep: number;
  error: boolean;
}) {
  return (
    <Stepper activeStep={activeStep}>
      <Step key={0}>
        <StepLabel
          optional={<Typography variant="caption">~20min</Typography>}
          error={error && activeStep === 0}
        >
          Cancelling swap
        </StepLabel>
      </Step>
      <Step key={1}>
        <StepLabel
          optional={<Typography variant="caption">~20min</Typography>}
          error={error && activeStep === 1}
        >
          Attempting recovery
        </StepLabel>
      </Step>
    </Stepper>
  );
}

export default function SwapStateStepper() {
  // TODO: There's no equivalent of this with Tauri yet.
  const currentSwapSpawnType = useAppSelector((s) => s.swap.spawnType);

  const swapState = useAppSelector((s) => s.swap.state);

  const [pathType, activeStep, error] = getActiveStep(swapState);

  // TODO: Fix this to work with Tauri
  // If the current swap is being manually cancelled and refund, we want to show the unhappy path even though the current state is not a "unhappy" state
  if (currentSwapSpawnType === SwapSpawnType.CANCEL_REFUND) {
    return <UnhappyPathStepper activeStep={0} error={error} />;
  }

  if (pathType === PathType.HAPPY_PATH) {
    return <HappyPathStepper activeStep={activeStep} error={error} />;
  }
  return <UnhappyPathStepper activeStep={activeStep} error={error} />;
}

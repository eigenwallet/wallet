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
  const processExited = state.curr.type === "Released";

  // If the swap is released we use the previous state to display the correct step
  const latestState = processExited ? prevState : state.curr;

  // If the swap is released but we do not have a previous state we fallback
  if(latestState === null) {
    return [PathType.HAPPY_PATH, 0, true];
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
    case "EncryptedSignatureSent":
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

    // Unhappy Path States

    // Step 1: Cancel timelock has expired. Waiting for cancel transaction to be published
    case "CancelTimelockExpired":
      return [PathType.UNHAPPY_PATH, 0, processExited];

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
      return exhaustiveGuard(latestState.type);
  }
}

function SwapStepper({ steps, activeStep, error }: {
  steps: Array<{ label: string; duration: string }>;
  activeStep: number;
  error: boolean;
}) {
  return (
    <Stepper activeStep={activeStep}>
      {steps.map((step, index) => (
        <Step key={index}>
          <StepLabel
            optional={<Typography variant="caption">{step.duration}</Typography>}
            error={error && activeStep === index}
          >
            {step.label}
          </StepLabel>
        </Step>
      ))}
    </Stepper>
  );
}

const HAPPY_PATH_STEP_LABELS = [
  { label: "Locking your BTC", duration: "~12min" },
  { label: "They lock their XMR", duration: "~18min" },
  { label: "They redeem the BTC", duration: "~2min" },
  { label: "Redeeming your XMR", duration: "~2min" },
];

const UNHAPPY_PATH_STEP_LABELS = [
  { label: "Cancelling swap", duration: "~1min" },
  { label: "Attempting recovery", duration: "~5min" },
];

export default function SwapStateStepper() {
  const swapState = useAppSelector((s) => s.swap.state);
  const [pathType, activeStep, error] = getActiveStep(swapState);

  const steps = pathType === PathType.HAPPY_PATH ? HAPPY_PATH_STEP_LABELS : UNHAPPY_PATH_STEP_LABELS;

  return <SwapStepper steps={steps} activeStep={activeStep} error={error} />;
}

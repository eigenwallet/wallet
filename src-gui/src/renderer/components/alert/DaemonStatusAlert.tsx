import { Box, Button, LinearProgress, makeStyles } from "@material-ui/core";
import { Alert } from "@material-ui/lab";
import { useNavigate } from "react-router-dom";
import { useAppSelector, usePendingBackgroundProcesses } from "store/hooks";
import { exhaustiveGuard } from "utils/typescriptUtils";
import { LoadingSpinnerAlert } from "./LoadingSpinnerAlert";
import { bytesToMb } from "utils/conversionUtils";
import { TauriBackgroundProgress, TauriContextStatusEvent } from "models/tauriModel";

const useStyles = makeStyles((theme) => ({
  innerAlert: {
    display: "flex",
    flexDirection: "column",
    gap: theme.spacing(2),
  },
}));

function AlertWithLinearProgress({ title, progress }: {
  title: string,
  progress: number | null,
}) {
  return <Alert severity="info">
    <Box style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
      {title}
      {(progress === null || progress === 0) ? (
        <LinearProgress variant="indeterminate" />
      ) : (
        <LinearProgress variant="determinate" value={progress} />
      )}
    </Box>
  </Alert>
}

function PartialInitStatus({ status }: {
  status: TauriBackgroundProgress,
  classes: ReturnType<typeof useStyles>
}) {
  if (status.progress.type === "Completed") {
    return null;
  }

  switch (status.componentName) {
    case "EstablishingTorCircuits":
      console.log("EstablishingTorCircuits", status.progress.content);
      return <AlertWithLinearProgress title="Establishing Tor circuits" progress={status.progress.content.frac * 100} />
    case "SyncingBitcoinWallet":
      const progressValue =
        status.progress.content?.type === "Known" ? 
        (status.progress.content?.content?.consumed / status.progress.content?.content?.total) * 100 : null;

      return <AlertWithLinearProgress title="Syncing internal Bitcoin wallet" progress={progressValue} />
    case "OpeningBitcoinWallet":
      return (
        <LoadingSpinnerAlert severity="info">
          Opening Bitcoin wallet
        </LoadingSpinnerAlert>
      );
    case "DownloadingMoneroWalletRpc":
      return <AlertWithLinearProgress title={`Downloading and verifying the Monero wallet RPC (${bytesToMb(status.progress.content.size).toFixed(2)} MB)`} progress={status.progress.content.progress} />
    case "OpeningMoneroWallet":
      return (
        <LoadingSpinnerAlert severity="info">
          Opening the Monero wallet
        </LoadingSpinnerAlert>
      );
    case "OpeningDatabase":
      return (
        <LoadingSpinnerAlert severity="info">
          Opening the local database
        </LoadingSpinnerAlert>
      );
    default:
      return null;
  }
}

export default function DaemonStatusAlert() {
  const classes = useStyles();
  const contextStatus = useAppSelector((s) => s.rpc.status);
  const navigate = useNavigate();

  if (contextStatus === null || contextStatus === TauriContextStatusEvent.NotInitialized) {
    return <LoadingSpinnerAlert severity="warning">Checking for available remote nodes</LoadingSpinnerAlert>;
  }

  switch (contextStatus) {
    case TauriContextStatusEvent.Initializing:
      return <LoadingSpinnerAlert severity="warning">Initializing the daemon</LoadingSpinnerAlert>;
    case TauriContextStatusEvent.Available:
      return <Alert severity="success">The daemon is running</Alert>;
    case TauriContextStatusEvent.Failed:
      return (
        <Alert
          severity="error"
          action={
            <Button
              size="small"
              variant="outlined"
              onClick={() => navigate("/help#daemon-control-box")}
            >
              View Logs
            </Button>
          }
        >
          The daemon has stopped unexpectedly
        </Alert>
      );
    default:
      return exhaustiveGuard(contextStatus);
  }
}

export function BackgroundProgressAlerts() {
  const backgroundProgress = usePendingBackgroundProcesses();
  const classes = useStyles();

  if (backgroundProgress.length === 0) {
    return null;
  }

  return backgroundProgress.map(([id, status]) => (
    <PartialInitStatus key={id} status={status} classes={classes} />
  ));
}
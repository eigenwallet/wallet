import { Box, makeStyles, Typography } from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { useLastDiscoveryProgress, useSettings } from "store/hooks";
import { Search } from "@material-ui/icons";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import { listSellersAtRendezvousPoint } from "renderer/rpc";
import { useAppDispatch } from "store/hooks";
import { discoveredMakersByRendezvous } from "store/features/makersSlice";
import { useState } from "react";
import { LinearProgress } from "@material-ui/core";
import { useRendezvousPointsProgress, useQuotesProgress } from "store/hooks";

const useStyles = makeStyles((theme) => ({
  title: {
    display: "flex",
    alignItems: "center",
    gap: theme.spacing(1),
  },
  button: {
    marginTop: theme.spacing(2),
  },
  progressContainer: {
    display: "flex",
    flexDirection: "column",
    gap: theme.spacing(1),
    marginBottom: theme.spacing(2),
  },
  progressLabel: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: theme.spacing(0.5),
  },
  mainContent: {
    display: "flex",
    flexDirection: "column",
    gap: theme.spacing(2),
  },
  statusText: {
    display: "flex",
    justifyContent: "space-between",
    marginTop: theme.spacing(0.5),
  }
}));

function ProgressIndicators() {
  const classes = useStyles();
  const discoveryProgress = useLastDiscoveryProgress();
  const rendezvousProgress = useRendezvousPointsProgress();
  const quotesProgress = useQuotesProgress();
  
  if (!discoveryProgress) return null;

  const { 
    total_rendezvous_points, 
    total_succeeded_rendezvous_points, 
    total_failed_rendezvous_points,
    total_quote_requests,
    total_succeeded_quote_requests,
    total_failed_quote_requests
  } = discoveryProgress;

  return (
    <Box className={classes.progressContainer}>
      <Box>
        <Box className={classes.progressLabel}>
          <Typography variant="body2">Discovering makers</Typography>
          <Typography variant="body2">
            {total_succeeded_rendezvous_points + total_failed_rendezvous_points} / {total_rendezvous_points}
          </Typography>
        </Box>
        <LinearProgress 
          variant="determinate" 
          value={rendezvousProgress || 0} 
        />
        <Box className={classes.statusText}>
          <Typography variant="caption" color="textSecondary">
            Success: {total_succeeded_rendezvous_points}
          </Typography>
          <Typography variant="caption" color="error">
            Failed: {total_failed_rendezvous_points}
          </Typography>
        </Box>
      </Box>
      
      <Box>
        <Box className={classes.progressLabel}>
          <Typography variant="body2">Connecting to makers</Typography>
          <Typography variant="body2">
            {total_succeeded_quote_requests + total_failed_quote_requests} / {total_quote_requests}
          </Typography>
        </Box>
        <LinearProgress 
          variant="determinate" 
          value={quotesProgress || 0} 
        />
        <Box className={classes.statusText}>
          <Typography variant="caption" color="textSecondary">
            Success: {total_succeeded_quote_requests}
          </Typography>
          <Typography variant="caption" color="error">
            Failed: {total_failed_quote_requests}
          </Typography>
        </Box>
      </Box>
    </Box>
  );
}

export default function DiscoveryBox() {
  const classes = useStyles();
  const rendezvousPoints = useSettings((s) => s.rendezvousPoints);
  const dispatch = useAppDispatch();
  const [discovering, setDiscovering] = useState(false);

  return (
    <InfoBox
      title={
        <Box className={classes.title}>
          Discover Makers
        </Box>
      }
      mainContent={
        <Box className={classes.mainContent}>
          <ProgressIndicators />
          <Typography variant="subtitle2">
            By connecting to rendezvous points run by volunteers, you can discover makers and then connect and swap with them in a decentralized manner.
            
            You have {rendezvousPoints.length} stored rendezvous {rendezvousPoints.length === 1 ? 'point' : 'points'} which we will connect to. We will also attempt to connect to peers which you have previously connected to.
          </Typography>
        </Box>
      }
      additionalContent={
        <PromiseInvokeButton
          variant="contained"
          color="primary"
          onInvoke={() => listSellersAtRendezvousPoint(rendezvousPoints)}
          onPendingChange={setDiscovering}
          onSuccess={({sellers}) => dispatch(discoveredMakersByRendezvous(sellers))}
          disabled={rendezvousPoints.length === 0 || discovering}
          startIcon={<Search />}
          className={classes.button}
          displayErrorSnackbar
        >
          Discover Makers
        </PromiseInvokeButton>
      }
      icon={null}
      loading={false}
    />
  );
} 
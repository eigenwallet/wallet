import { Box, Tooltip } from "@mui/material";
import makeStyles from '@mui/styles/makeStyles';
import GitHubIcon from "@mui/icons-material/GitHub";
import DaemonStatusAlert, { BackgroundProgressAlerts } from "../alert/DaemonStatusAlert";
import FundsLeftInWalletAlert from "../alert/FundsLeftInWalletAlert";
import MoneroWalletRpcUpdatingAlert from "../alert/MoneroWalletRpcUpdatingAlert";
import UnfinishedSwapsAlert from "../alert/UnfinishedSwapsAlert";
import LinkIconButton from "../icons/LinkIconButton";
import BackgroundRefundAlert from "../alert/BackgroundRefundAlert";
import MatrixIcon from "../icons/MatrixIcon";
import { MenuBook } from "@mui/icons-material";

const useStyles = makeStyles((theme) => ({
  outer: {
    display: "flex",
    flexDirection: "column",
    padding: theme.spacing(1),
    gap: theme.spacing(1),
  },
  linksOuter: {
    display: "flex",
    justifyContent: "space-evenly",
  },
}));

export default function NavigationFooter() {
  const classes = useStyles();

  return (
    <Box className={classes.outer}>
      <FundsLeftInWalletAlert />
      <UnfinishedSwapsAlert />
      <BackgroundRefundAlert />
      <DaemonStatusAlert />
      <BackgroundProgressAlerts />
      <MoneroWalletRpcUpdatingAlert />
      <Box className={classes.linksOuter}>
        <Tooltip title="Check out the GitHub repository">
          <span>
            <LinkIconButton url="https://github.com/UnstoppableSwap/core">
              <GitHubIcon />
            </LinkIconButton>
          </span>
        </Tooltip>
        <Tooltip title="Join the Matrix room">
          <span>
            <LinkIconButton url="https://matrix.to/#/#unstoppableswap-space:matrix.org">
              <MatrixIcon />
            </LinkIconButton>
          </span>
        </Tooltip>
        <Tooltip title="Read our official documentation">
          <span>
            <LinkIconButton url="https://docs.unstoppableswap.net">
              <MenuBook />
            </LinkIconButton>
          </span>
        </Tooltip>
      </Box>
    </Box>
  );
}

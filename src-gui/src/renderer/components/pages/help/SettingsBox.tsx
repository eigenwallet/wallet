import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableRow,
  Typography,
  IconButton,
  Box,
  makeStyles,
  Tooltip,
  Switch,
  Select,
  MenuItem,
  TableHead,
  Paper,
  Button,
  Dialog,
  DialogContent,
  DialogActions,
  DialogTitle,
  useTheme,
  DialogContentText,
} from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import {
  resetSettings,
  setElectrumRpcUrl,
  setMoneroNodeUrl,
} from "store/features/settingsSlice";
import { useAppDispatch, useSettings } from "store/hooks";
  addNode,
  Blockchain,
  moveUpNode,
  Network,
  removeNode,
  resetSettings,
  setTheme,
} from "store/features/settingsSlice";
import { useAppDispatch, useAppSelector, useNodes, useSettings } from "store/hooks";
import ValidatedTextField from "renderer/components/other/ValidatedTextField";
import RefreshIcon from "@material-ui/icons/Refresh";
import HelpIcon from '@material-ui/icons/HelpOutline';
import { ReactNode, useEffect, useState, useMemo } from "react";
import { Theme } from "renderer/components/theme";
import { getNetwork } from "store/config";
import { Add, ArrowUpward, Delete, Edit, HourglassEmpty } from "@material-ui/icons";
import { updateAllNodeStatuses } from "renderer/rpc";

const PLACEHOLDER_ELECTRUM_RPC_URL = "ssl://blockstream.info:700";
const PLACEHOLDER_MONERO_NODE_URL = "http://xmr-node.cakewallet.com:18081";

const useStyles = makeStyles((theme) => ({
  title: {
    display: "flex",
    alignItems: "center",
    gap: theme.spacing(1),
  }
}));

export default function SettingsBox() {
  const dispatch = useAppDispatch();
  const classes = useStyles();
  const theme = useTheme();
  const [resetModalOpen, setResetModalOpen] = useState(false);

  return (
    <InfoBox
      title={
        <Box className={classes.title}>
          Settings
          <IconButton
          size="small"
          onClick={() => {
            dispatch(resetSettings());
          }}
        >
          <RefreshIcon />
        </IconButton>
        </Box>
      }
      additionalContent={
        <>
          <TableContainer>
            <Table>
              <TableBody>
                <ElectrumRpcUrlSetting />
                <MoneroNodeUrlSetting />
                <ThemeSetting />
              </TableBody>
            </Table>
          </TableContainer>
          <Box mt={theme.spacing(0.1)} />
          <Button 
            onClick={() => {
              setResetModalOpen(true);
            }}
            variant="outlined"
          >
            Reset Settings
          </Button>
          <Dialog
            open={resetModalOpen}
            onClose={() => setResetModalOpen(false)}
          >
            <DialogTitle>Reset Settings</DialogTitle>
            <DialogContent>
              <DialogContentText>
                Are you sure you want to reset all settings to their default values? This cannot be undone.
              </DialogContentText>
            </DialogContent>
            <DialogActions>
              <Button onClick={() => setResetModalOpen(false)}>
                Cancel
              </Button>
              <Button 
                onClick={() => {
                  dispatch(resetSettings());
                  setResetModalOpen(false);
                }}
                color="primary"
                variant="contained"
              >
                Reset
              </Button>
            </DialogActions>
          </Dialog>
        </>
      }
      mainContent={
        <Typography variant="subtitle2">
          Customize the settings of the GUI. 
          Some of these require a restart to take effect.
        </Typography>
      }
      icon={null}
      loading={false}
    />
  );
}

// URL validation function, forces the URL to be in the format of "protocol://host:port/"
function isValidUrl(url: string, allowedProtocols: string[]): boolean {
  const urlPattern = new RegExp(`^(${allowedProtocols.join("|")})://[^\\s]+:\\d+/?$`);
  return urlPattern.test(url);
}

function ElectrumRpcUrlSetting() {
  const electrumRpcUrl = useSettings((s) => s.electrum_rpc_url);
  const dispatch = useAppDispatch();

  const [tableVisible, setTableVisible] = useState(false);

  function isValid(url: string): boolean {
    return isValidUrl(url, ["ssl", "tcp"]);
  }

  return (
      <TableRow>
        <TableCell>
          <SettingLabel label="Custom Electrum RPC URL" tooltip="This is the URL of the Electrum server that the GUI will connect to. It is used to sync Bitcoin transactions. If you leave this field empty, the GUI will choose from a list of known servers at random." />
      </TableCell>
      <TableCell>
        <IconButton 
            onClick={() => setTableVisible(true)}
          >
            {<Edit />}
          </IconButton>
        </TableCell>
          {tableVisible ? <NodeTableModal
            open={tableVisible}
            onClose={() => setTableVisible(false)}
            network={network}
            blockchain={Blockchain.Bitcoin}
            isValid={isValid}
            placeholder={PLACEHOLDER_ELECTRUM_RPC_URL}
          /> : <></>}
    </TableRow>
  );
}

function SettingLabel({ label, tooltip }: { label: ReactNode, tooltip: string | null }) {
  return <Box style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
    <Box>
      {label}
    </Box>
    <Tooltip title={tooltip}>
      <IconButton size="small">
        <HelpIcon />
      </IconButton>
    </Tooltip>
  </Box>
}

function MoneroNodeUrlSetting() {
  const moneroNodeUrl = useSettings((s) => s.monero_node_url);
  const dispatch = useAppDispatch();

  function isValid(url: string): boolean {
    return isValidUrl(url, ["http"]);
  }

  const [tableVisible, setTableVisible] = useState(false);

  return (
    <TableRow>
      <TableCell>
       <SettingLabel label="Custom Monero Node URL" tooltip="This is the URL of the Monero node that the GUI will connect to. Ensure the node is listening for RPC connections over HTTP. If you leave this field empty, the GUI will choose from a list of known nodes at random." />
      </TableCell>
      <TableCell>  
        <IconButton 
          onClick={() => setTableVisible(!tableVisible)}
        >
          <Edit /> 
        </IconButton>
      </TableCell>
      <TableCell>
        {tableVisible ? <NodeTableModal
          open={tableVisible}
          onClose={() => setTableVisible(false)}
          network={network}
          blockchain={Blockchain.Monero}
          isValid={isValid}
          onValidatedChange={(value) => {
            dispatch(setMoneroNodeUrl(value));
          }}
          fullWidth
          placeholder={PLACEHOLDER_MONERO_NODE_URL}
        /> : <></>}
      </TableCell>
    </TableRow>
  );
}

function ThemeSetting() {
  const theme = useAppSelector((s) => s.settings.theme);
  const dispatch = useAppDispatch();

  return (
    <TableRow>
      <TableCell>
        <SettingLabel label="Theme" tooltip="This is the theme of the GUI." />
      </TableCell>
      <TableCell>
        <Select 
          value={theme} 
          onChange={(e) => dispatch(setTheme(e.target.value as Theme))}
          variant="outlined"
        >
          {/** Create an option for each theme variant */}
          {Object.values(Theme).map((themeValue) => (
            <MenuItem key={themeValue} value={themeValue}>
              {themeValue.charAt(0).toUpperCase() + themeValue.slice(1)}
            </MenuItem>
          ))}
        </Select>
      </TableCell>
    </TableRow>
  );
}

function NodeTableModal({
  open,
  onClose,
  network,
  isValid,
  placeholder,
  blockchain
}: {
  network: Network;
  blockchain: Blockchain;
  isValid: (url: string) => boolean;
  placeholder: string;
  open: boolean;
  onClose: () => void;
}) {
  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>Available Nodes</DialogTitle>
      <DialogContent>
        <Typography variant="subtitle2">
          When the daemon is started, it will attempt to connect to the first {blockchain} node in this list.
          If you leave this field empty, it will choose from a list of known nodes at random.
        </Typography>
        <NodeTable network={network} blockchain={blockchain} isValid={isValid} placeholder={placeholder} />
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} size="large">Close</Button>
      </DialogActions>
    </Dialog>
  )
}

function NodeTable({
  network,
  blockchain,
  isValid,
  placeholder,
}: {
  network: Network,
  blockchain: Blockchain,
  isValid: (url: string) => boolean,
  placeholder: string,
}) {
  const availableNodes = useSettings((s) => s.nodes[network][blockchain]);
  const currentNode = availableNodes[0];
  const statuses = useNodes((s) => s.nodes);
  console.log(`Statuses`, statuses);
  const dispatch = useAppDispatch();
  const theme = useTheme();
  const circle = (color: string) => <svg width="12" height="12" viewBox="0 0 12 12">
    <circle cx="6" cy="6" r="6" fill={color} />
  </svg>;

  const statusIcon = useMemo(() => (node: string) => {
    switch (statuses[blockchain][node]) {
      case true:
        return <Tooltip title={"This node is available and responding to RPC requests"}>
          {circle(theme.palette.success.dark)}
        </Tooltip>;
      case false:
        return <Tooltip title={"This node is not available or not responding to RPC requests"}>
          {circle(theme.palette.error.dark)}
        </Tooltip>;
      default:
        console.log(`Unknown status for node ${node}: ${statuses[node]}`);
        return <Tooltip title={"The status of this node is currently unknown"}>
          <HourglassEmpty />
        </Tooltip>;
    }
  }, [statuses]);

  const [newNode, setNewNode] = useState("");

  const addNewNode = () => {
    dispatch(addNode({ network, type: blockchain, node: newNode }));
    setNewNode("");
  }

  useEffect(() => {
    updateAllNodeStatuses();
    
    const interval = setInterval(() => {
      updateAllNodeStatuses();
    }, 15_000);

    return () => clearInterval(interval);
  }, []);

  return (
    <TableContainer component={Paper} style={{ marginTop: '1rem' }} elevation={0}>
      <Table size="small">
        <TableHead>
          <TableRow>
            <TableCell align="center">Node URL</TableCell>
            <TableCell align="center">Status</TableCell>
            <TableCell align="center">Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {availableNodes.map((node, index) => (
            <TableRow key={index}>
              <TableCell>
                <Typography variant="overline">{node}</Typography>
              </TableCell>
              <TableCell align="center">{statusIcon(node)}</TableCell>
              <TableCell>
                <Tooltip title={"Remove this node from your list"}>
                  <IconButton 
                    onClick={() => {
                      dispatch(removeNode({ network, type: blockchain, node }));
                    }}
                  >
                    <Delete />
                  </IconButton>
                </Tooltip>
                { currentNode !== node ? <Tooltip title={"Move this node to the top of the list"}>
                  <IconButton onClick={async () => {
                    while (currentNode !== node) {
                      dispatch(moveUpNode({ network, type: blockchain, node }));
                      await new Promise((resolve) => setTimeout(resolve, 50));
                    } 
                  }}>
                    <ArrowUpward />
                  </IconButton>
                </Tooltip> : <></>}
              </TableCell>
            </TableRow>
          ))}
          <TableRow key={-1}>
            <TableCell>
              <ValidatedTextField
                label="Add a new node"
                value={newNode}
                onValidatedChange={setNewNode}
                placeholder={placeholder}
                fullWidth
                allowEmpty 
                isValid={isValid}
                variant="outlined"
                onKeyUp={(e) => {
                  if (e.key === 'Enter' && newNode)
                    addNewNode();
                }}
              />
            </TableCell>
            <TableCell></TableCell>
            <TableCell>
              <Tooltip title={"Add this node to your list"}>
                <IconButton onClick={addNewNode} disabled={availableNodes.includes(newNode)}>
                  <Add />
                </IconButton>
              </Tooltip>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </TableContainer>
  )
}
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
} from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import {
  resetSettings,
  setElectrumRpcUrl,
  setMoneroNodeUrl,
} from "store/features/settingsSlice";
import { useAppDispatch, useSettings } from "store/hooks";
import ValidatedTextField from "renderer/components/other/ValidatedTextField";
import RefreshIcon from "@material-ui/icons/Refresh";
import HelpIcon from '@material-ui/icons/HelpOutline';
import { ReactNode, useState } from "react";
import { Theme } from "renderer/components/theme";
import { getNetwork } from "store/config";
import { Add, Check, Delete, Edit, PlusOne, VisibilityOffRounded, VisibilityRounded } from "@material-ui/icons";

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
        <TableContainer>
          <Table>
            <TableBody>
              <ElectrumRpcUrlSetting />
              <MoneroNodeUrlSetting />
            </TableBody>
          </Table>
        </TableContainer>
      }
      mainContent={
        <Typography variant="subtitle2">
          Some of these settings require a restart to take effect.
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
            onClick={() => setTableVisible(!tableVisible)}
          >
            {tableVisible ? <VisibilityOffRounded /> : <VisibilityRounded />}
          </IconButton>
        </TableCell>
        <TableCell>
          {tableVisible ? <NodeTable
            blockchain={Blockchain.Bitcoin}
            isValid={isValid}
            placeholder={PLACEHOLDER_ELECTRUM_RPC_URL}
          /> : <></>}
        </TableCell>
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
          {tableVisible ? <VisibilityOffRounded /> : <VisibilityRounded /> } 
        </IconButton>
      </TableCell>
      <TableCell>
        {tableVisible ? <NodeTable
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
        <Select 
          value={theme} 
          onChange={(e) => dispatch(setTheme(e.target.value as Theme))}
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

function NodeTable({
  blockchain,
  isValid,
  placeholder,
}: {
  blockchain: Blockchain,
  isValid: (url: string) => boolean,
  placeholder: string,
}) {
  const [currentNode, setNode] = blockchain == Blockchain.Bitcoin ? 
    [useSettings((s) => s.tauriSettings.electrum_rpc_url), setElectrumRpcUrl]
    : [useSettings((s) => s.tauriSettings.monero_node_url), setMoneroNodeUrl];
  const availableNodes = useSettings((s) => s.nodes[blockchain]);
  const dispatch = useAppDispatch();

  const [newNode, setNewNode] = useState("");

  return (
    <TableContainer component={Paper} style={{ marginTop: '1rem' }}>
      <Table size="small">
        <TableHead>
          <TableRow>
            <TableCell>Active</TableCell>
            <TableCell>Available Nodes</TableCell>
            <TableCell>Status</TableCell>
            <TableCell>Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {availableNodes.map((node, index) => (
            <TableRow key={index}>
              <TableCell>{currentNode === node ? <Check color="secondary"/> : <></>}</TableCell>
              <TableCell>{node}</TableCell>
              <TableCell>Hallo</TableCell>
              <TableCell>
                <IconButton onClick={() => {
                  dispatch(setNode(node));
                }}>
                  <Check />
                </IconButton>
                <IconButton onClick={() => {
                  dispatch(removeNode({ type: blockchain, node }));
                }}>
                  <Delete />
                </IconButton>
              </TableCell>
            </TableRow>
          ))}
          <TableRow key={-1}>
            <TableCell></TableCell>
            <TableCell>
              <ValidatedTextField
                label="new node"
                value={newNode}
                onValidatedChange={setNewNode}
                placeholder={placeholder}
                fullWidth
                allowEmpty 
                isValid={isValid}
              />
            </TableCell>
            <TableCell></TableCell>
            <TableCell>
              <IconButton onClick={() => {
                dispatch(addNode({ type: blockchain, node: newNode }));
                setNewNode("");
              }}>
                <Add />
              </IconButton>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </TableContainer>
  )
}
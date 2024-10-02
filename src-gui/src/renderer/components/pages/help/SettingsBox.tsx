import React from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableRow,
  MenuItem,
  Select,
  Typography,
} from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import {
  setBitcoinConfirmationTarget,
  setElectrumRpcUrl,
  setMoneroNodeUrl,
} from "store/features/settingsSlice";
import { useAppDispatch, useAppSelector } from "store/hooks";
import ValidatedTextField from "renderer/components/other/ValidatedTextField";

export default function SettingsBox() {
  const bitcoinConfirmationTarget = useAppSelector(
    (s) => s.settings.bitcoin_confirmation_target,
  );
  const electrumRpcUrl = useAppSelector((s) => s.settings.electrum_rpc_url);
  const moneroNodeUrl = useAppSelector((s) => s.settings.monero_node_url);
  const dispatch = useAppDispatch();

  // URL validation function, forces the URL to be in the format of "protocol://host:port/path"
  function isValidUrl(string) {
    const pattern = /^(https?|ssl|tcp):\/\/[^\/:\s]+:\d+(\/[^\s]*)?$/i;
    return pattern.test(string);
  }

  return (
    <InfoBox
      title="Settings"
      additionalContent={
        <TableContainer>
          <Table>
            <TableBody>
              <TableRow>
                <TableCell>Bitcoin confirmation target</TableCell>
                <TableCell>
                  <Select
                    value={bitcoinConfirmationTarget}
                    onChange={(e) => {
                      dispatch(
                        setBitcoinConfirmationTarget(Number(e.target.value)),
                      );
                    }}
                  >
                    {[1, 2, 3].map((target) => (
                      <MenuItem key={target} value={target}>
                        {target} block{target > 1 ? "s" : ""}
                      </MenuItem>
                    ))}
                  </Select>
                </TableCell>
              </TableRow>
              <TableRow>
                <TableCell>Custom Electrum RPC URL</TableCell>
                <TableCell>
                  <ValidatedTextField
                    label="Electrum RPC URL"
                    value={electrumRpcUrl}
                    isValid={isValidUrl}
                    onValidatedChange={(value) => {
                      dispatch(setElectrumRpcUrl(value));
                    }}
                    fullWidth
                    placeholder="ssl://blockstream.info:700"
                    allowEmpty
                  />
                </TableCell>
              </TableRow>
              <TableRow>
                <TableCell>Custom Monero Node URL</TableCell>
                <TableCell>
                  <ValidatedTextField
                    label="Monero Node URL"
                    value={moneroNodeUrl}
                    isValid={isValidUrl}
                    onValidatedChange={(value) => {
                      dispatch(setMoneroNodeUrl(value));
                    }}
                    fullWidth
                    placeholder="http://xmr-node.cakewallet.com:18081"
                    allowEmpty
                  />
                </TableCell>
              </TableRow>
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

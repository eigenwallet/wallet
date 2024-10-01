import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableRow,
  TextField,
} from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { setBitcoinConfirmationTarget, setElectrumRpcUrl, setMoneroNodeUrl } from "store/features/settingsSlice";
import { useAppDispatch, useAppSelector } from "store/hooks";
import { MenuItem, Select } from "@material-ui/core";

export default function SettingsInfoBox() {
  const bitcoinConfirmationTarget = useAppSelector(
    (s) => s.settings.bitcoin_confirmation_target,
  );
  const electrumRpcUrl = useAppSelector((s) => s.settings.electrum_rpc_url);
  const moneroNodeUrl = useAppSelector((s) => s.settings.monero_node_url);
  const dispatch = useAppDispatch();

  return (
    <InfoBox
      title="Settings"
      mainContent={
        <TableContainer>
          <Table>
            <TableBody>
              <TableRow>
                <TableCell>Bitcoin confirmation target</TableCell>
                <TableCell>
                  <Select
                    value={bitcoinConfirmationTarget}
                    onChange={(e) => {
                      dispatch(setBitcoinConfirmationTarget(Number(e.target.value)));
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
                <TableCell>Electrum RPC URL</TableCell>
                <TableCell>
                  <TextField
                  value={electrumRpcUrl}
                  onChange={(e) => {
                    dispatch(setElectrumRpcUrl(e.target.value));
                  }}
                  fullWidth
                  />
                </TableCell>
                </TableRow>
                <TableRow>
                <TableCell>Monero Node URL</TableCell>
                <TableCell>
                  <TextField
                  value={moneroNodeUrl}
                  onChange={(e) => {
                    dispatch(setMoneroNodeUrl(e.target.value));
                  }}
                  fullWidth
                  />
                </TableCell>
                </TableRow>
            </TableBody>
          </Table>
        </TableContainer>
      }
      additionalContent={null}
      icon={null}
      loading={false}
    />
  );
}

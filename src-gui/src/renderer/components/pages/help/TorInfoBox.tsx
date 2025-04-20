import { Box, Switch, Table, TableCell, TableRow, Typography } from "@material-ui/core";
import { useSettings } from "store/hooks";
import InfoBox from "../../modal/swap/InfoBox";
import { useDispatch } from "react-redux";
import { setTorEnabled } from "store/features/settingsSlice";

export default function TorInfoBox() {
  const dispath = useDispatch();
  const torEnabled = useSettings((settings) => settings.enableTor)
  const handleChange = _ => dispath(setTorEnabled(!torEnabled));
  const labelText = (state: boolean) => state === true ? "Tor enabled" : "Tor disabled";

  return (
    <InfoBox
      title="Tor (The Onion Router)"
      mainContent={
        <Box
          style={{
            width: "100%",
            display: "flex",
            flexDirection: "column",
            gap: "8px",
          }}
        >
          <Typography variant="subtitle2">
            Tor is a network that allows you to anonymously connect to the
            internet. It is a free and open network operated by
            volunteers. If Tor is running, all peer-to-peer traffic will be routed through it and
            the maker will not be able to see your IP address.

            Requires a restart to take effect.
          </Typography>
        </Box>
      }
      additionalContent={
        <Table size="small">
          <TableRow>
            <TableCell>
              {labelText(torEnabled)}
            </TableCell>
            <TableCell>
              <Switch checked={torEnabled} onChange={handleChange} color="primary" />
            </TableCell>
          </TableRow>
        </Table>
      }
      icon={null}
      loading={false}
    />
  );
}

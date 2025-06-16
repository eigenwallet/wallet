import { Box, Radio, Typography } from "@mui/material";
import Avatar from "boring-avatars";
import { AccessTimeOutlined as ClockIcon } from "@mui/icons-material";
import { MonetizationOnOutlined as MoneyIcon } from "@mui/icons-material";
import { CurrencyBitcoinOutlined as BitcoinIcon } from "@mui/icons-material";
import IconChip from "../IconChip";
import { ExtendedMakerStatus } from "models/apiModel";

export default function MakerOfferItem({
  maker,
  onSelect,
  checked,
}: {
  maker: ExtendedMakerStatus;
  onSelect: (maker: ExtendedMakerStatus) => void;
  checked: boolean;
}) {
  if (maker === null || maker === undefined) {
    return null;
  }

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        gap: 2,
        border: "1px solid",
        borderColor: "divider",
        borderRadius: 2,
        padding: 2,
        width: "100%",
        cursor: checked ? "default" : "pointer",
      }}
      onClick={() => {
        if (!checked) {
          onSelect(maker);
        }
      }}
    >
      <Box sx={{ width: "min-content", height: "min-content", flexShrink: 0 }}>
        <Radio checked={checked} />
      </Box>
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          gap: 1,
          flexShrink: 1,
          minWidth: 0,
        }}
      >
        <Typography variant="h4" noWrap>
          {maker.multiAddr}
        </Typography>
        <Box
          sx={{
            display: "flex",
            flexDirection: "row",
            gap: 1,
            marginBottom: 2,
          }}
        >
          <Box
            sx={{
              width: "min-content",
              height: "min-content",
              aspectRatio: 1,
              borderRadius: 1,
              overflow: "hidden",
            }}
          >
            <Avatar
              size={25}
              name={maker.peerId}
              variant="marble"
              colors={["#92A1C6", "#146A7C", "#F0AB3D", "#C271B4", "#C20D90"]}
              square
            />
          </Box>
          <Typography variant="body1" noWrap>
            {maker.peerId}
          </Typography>
        </Box>
        <Box
          sx={{
            display: "flex",
            flexDirection: "row",
            gap: 1,
          }}
        >
          <IconChip icon={<ClockIcon />} color="primary.main">
            active for{" "}
            <Typography
              sx={{
                fontWeight: 800,
                fontSize: 12,
              }}
            >
              10 minutes
            </Typography>
          </IconChip>
          <IconChip icon={<MoneyIcon />} color="primary.main">
            <Typography
              sx={{
                fontWeight: 800,
                fontSize: 12,
              }}
            >
              0.12 %
            </Typography>{" "}
            fee
          </IconChip>
          <IconChip icon={<BitcoinIcon />} color="primary.main">
            <Typography
              sx={{
                fontWeight: 800,
                fontSize: 12,
              }}
            >
              0.00003 – 0.00500
            </Typography>
          </IconChip>
        </Box>
      </Box>
    </Box>
  );
}

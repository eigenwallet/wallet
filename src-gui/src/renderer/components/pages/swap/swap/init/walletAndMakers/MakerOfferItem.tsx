import { Box, Typography } from "@mui/material";
import Avatar from "boring-avatars";
import { ApprovalRequest } from "models/tauriModel";

export default function MakerOfferItem({
  makerApproval,
  selectedMakerRequestId,
  onSelect,
}: {
  makerApproval: ApprovalRequest;
  selectedMakerRequestId: string | null;
  onSelect: (requestId: string) => void;
}) {
  const isSelected =
    selectedMakerRequestId === makerApproval.content.request_id;

  if (makerApproval.content.details.type !== "SelectMaker") {
    return null;
  }

  const { maker, btc_amount_to_swap } = makerApproval.content.details.content;
  const { multiaddr, peer_id, quote, version } = maker;

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        gap: 2,
        borderRadius: 2,
        padding: 2,
        width: "100%",
        justifyContent: "space-between",
        alignItems: "center",
        backgroundColor: isSelected ? "primary.light" : "background.default",
        cursor: "pointer",
        border: "1px solid",
        borderColor: isSelected ? "primary.main" : "divider",
        "&:hover": {
          backgroundColor: isSelected ? "primary.light" : "action.hover",
        },
      }}
      onClick={() => onSelect(makerApproval.content.request_id)}
    >
      <Box
        sx={{
          display: "flex",
          flexDirection: "row",
          gap: 2,
        }}
      >
        <Avatar
          size={40}
          name={peer_id}
          variant="marble"
          colors={["#92A1C6", "#146A7C", "#F0AB3D", "#C271B4", "#C20D90"]}
        />
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            gap: 1,
          }}
        >
          <Typography variant="body1" sx={{ maxWidth: "200px" }} noWrap>
            {multiaddr}
          </Typography>
          <Typography variant="body1" sx={{ maxWidth: "200px" }} noWrap>
            {peer_id}
          </Typography>
          <Box
            sx={{
              display: "flex",
              flexDirection: "row",
              gap: 1,
            }}
          >
            <Typography variant="body1">{quote.price} XMR/BTC</Typography>
            <Typography variant="body1">
              {quote.min_quantity} - {quote.max_quantity} XMR
            </Typography>
            <Typography variant="body1">Version: {version}</Typography>
          </Box>
        </Box>
      </Box>
      <Box sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
        <Typography variant="body1">{btc_amount_to_swap} XMR</Typography>
      </Box>
    </Box>
  );
}

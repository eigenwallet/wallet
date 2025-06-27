import {
  DialogContent,
  DialogActions,
  Typography,
  Box,
  Divider,
} from "@mui/material";
import {
  PendingLockBitcoinApprovalRequest,
  TauriSwapProgressEventContent,
} from "models/tauriModelExt";
import { useEffect, useState } from "react";
import { useActiveSwapId, usePendingLockBitcoinApproval } from "store/hooks";
import { resolveApproval } from "renderer/rpc";
import { SatsAmount } from "renderer/components/other/Units";
import CircularProgressWithSubtitle from "renderer/components/pages/swap/swap/components/CircularProgressWithSubtitle";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import { Check as CheckIcon } from "@mui/icons-material";

function useActiveLockBitcoinApprovalRequest(): PendingLockBitcoinApprovalRequest | null {
  const approvals = usePendingLockBitcoinApproval();
  const activeSwapId = useActiveSwapId();

  return (
    approvals?.find(
      (r) => r.content.details.content.swap_id === activeSwapId,
    ) || null
  );
}

export default function Offer({
  btc_lock_amount,
}: TauriSwapProgressEventContent<"SwapSetupInflight">) {
  const request = useActiveLockBitcoinApprovalRequest();

  const [timeLeft, setTimeLeft] = useState<number>(0);

  const expiresAtMs = request?.content.expiration_ts * 1000 || 0;

  useEffect(() => {
    const tick = () => {
      const remainingMs = Math.max(expiresAtMs - Date.now(), 0);
      setTimeLeft(Math.ceil(remainingMs / 1000));
    };

    tick();
    const id = setInterval(tick, 250);
    return () => clearInterval(id);
  }, [expiresAtMs]);

  if (!request) {
    return (
      <CircularProgressWithSubtitle
        description={
          <>
            Negotiating offer for <SatsAmount amount={btc_lock_amount} />
          </>
        }
      />
    );
  }

  const { btc_network_fee, xmr_receive_amount } =
    request.content.details.content;

  return (
    <>
      <Box>
        <Typography variant="body1">
          Review and confirm your swap offer
        </Typography>
        <Box
          sx={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: 2,
            mt: 3,
          }}
        >
          <Typography variant="body1" sx={{ fontWeight: "bold" }}>
            You send
          </Typography>
          <Typography
            sx={{ textAlign: "right", fontWeight: "bold" }}
            variant="body1"
          >
            <SatsAmount amount={btc_lock_amount} />
          </Typography>
          <Divider sx={{ gridColumn: "span 2" }} />

          <Typography variant="body1" sx={{ fontWeight: "bold" }}>
            Bitcoin network fees
          </Typography>
          <Typography
            sx={{ textAlign: "right", fontWeight: "bold" }}
            variant="body1"
          >
            <SatsAmount amount={btc_network_fee} />
          </Typography>

          <Divider sx={{ gridColumn: "span 2" }} />

          <Typography variant="body1" sx={{ fontWeight: "bold" }}>
            You receive
          </Typography>
          <Typography
            sx={{ textAlign: "right", fontWeight: "bold" }}
            variant="body1"
          >
            <SatsAmount amount={xmr_receive_amount} />
          </Typography>
        </Box>
      </Box>

      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <PromiseInvokeButton
          variant="text"
          size="large"
          sx={(theme) => ({ color: theme.palette.text.secondary })}
          onInvoke={() => resolveApproval(request.content.request_id, false)}
          displayErrorSnackbar
          requiresContext
        >
          Deny
        </PromiseInvokeButton>
        <Box
          sx={{
            display: "flex",
            flexDirection: "row",
            alignItems: "center",
            gap: 2,
          }}
        >
          <Typography variant="body1" sx={{ fontWeight: "bold" }}>
            expires in {timeLeft}
          </Typography>
          <PromiseInvokeButton
            variant="contained"
            color="primary"
            size="large"
            onInvoke={() => resolveApproval(request.content.request_id, true)}
            displayErrorSnackbar
            requiresContext
            endIcon={<CheckIcon />}
          >
            Confirm
          </PromiseInvokeButton>
        </Box>
      </Box>
    </>
  );
}

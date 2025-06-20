import { Box, Typography, Paper, Divider } from "@mui/material";
import { TauriSwapProgressEventContent } from "models/tauriModelExt";
import BitcoinIcon from "../../../../icons/BitcoinIcon";
import { SatsAmount, MoneroBitcoinExchangeRate } from "../../../../other/Units";
import DepositAddressInfoBox from "../../DepositAddressInfoBox";
import { Alert } from "@mui/material";
import { usePendingSelectMakerApproval } from "store/hooks";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import { resolveApproval } from "renderer/rpc";
import IdentIcon from "renderer/components/icons/IdentIcon";
import TruncatedText from "renderer/components/other/TruncatedText";
import { satsToBtc } from "utils/conversionUtils";

export default function WaitingForBtcDepositPage({
  deposit_address,
  min_bitcoin_lock_tx_fee,
  max_giveable,
}: TauriSwapProgressEventContent<"WaitingForBtcDeposit">) {
  const pendingSelectMakerApprovals = usePendingSelectMakerApproval();

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
      {/* Deposit Address Section */}
      <DepositAddressInfoBox
        title="Bitcoin Deposit Address"
        address={deposit_address}
        icon={<BitcoinIcon />}
        additionalContent={null}
      />

      {/* Balance and Fee Section */}
      <Paper variant="outlined" sx={{ p: 2 }}>
        <Typography variant="h6" gutterBottom>
          Wallet Information
        </Typography>
        <Box sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
          <Typography variant="body1">
            <strong>Available Balance:</strong>{" "}
            <SatsAmount amount={max_giveable} />
          </Typography>
          <Typography variant="body1">
            <strong>Network Fee:</strong> ≈{" "}
            <SatsAmount amount={min_bitcoin_lock_tx_fee} />
          </Typography>
        </Box>

        <Alert severity="info" sx={{ mt: 2 }}>
          Please do not use replace-by-fee on your deposit transaction. You'll
          need to start a new swap if you do.
        </Alert>
      </Paper>

      {/* Offers Section */}
      {pendingSelectMakerApprovals.length > 0 && (
        <Box>
          <Typography variant="h6" gutterBottom>
            Available Offers (
            {(() => {
              // Group by peer_id and keep only the latest offer for each peer
              const latestOffersByPeer = new Map();
              pendingSelectMakerApprovals.forEach((approval) => {
                const peerId = approval.content.details.content.maker.peer_id;
                const expirationTime = approval.content.expiration_ts;

                if (
                  !latestOffersByPeer.has(peerId) ||
                  expirationTime >
                    latestOffersByPeer.get(peerId).content.expiration_ts
                ) {
                  latestOffersByPeer.set(peerId, approval);
                }
              });
              return latestOffersByPeer.size;
            })()}
            )
          </Typography>

          <Box sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
            {(() => {
              // Group by peer_id and keep only the latest offer for each peer
              const latestOffersByPeer = new Map();
              pendingSelectMakerApprovals.forEach((approval) => {
                const peerId = approval.content.details.content.maker.peer_id;
                const expirationTime = approval.content.expiration_ts;

                if (
                  !latestOffersByPeer.has(peerId) ||
                  expirationTime >
                    latestOffersByPeer.get(peerId).content.expiration_ts
                ) {
                  latestOffersByPeer.set(peerId, approval);
                }
              });

              return Array.from(latestOffersByPeer.values());
            })().map((makerApproval, index, filteredApprovals) => {
              const { request_id, details } = makerApproval.content;
              const { maker } = details.content;

              return (
                <Box key={request_id}>
                  <Box
                    sx={{
                      display: "flex",
                      alignItems: "center",
                      gap: 2,
                      py: 1.5,
                      px: 2,
                      backgroundColor: "background.default",
                      borderRadius: 1,
                      border: "1px solid",
                      borderColor: "divider",
                    }}
                  >
                    {/* Icon and ID */}
                    <Box
                      sx={{
                        display: "flex",
                        alignItems: "center",
                        gap: 1.5,
                        flex: 1,
                      }}
                    >
                      <IdentIcon value={maker.peer_id} size="2rem" />
                      <Box sx={{ minWidth: 0 }}>
                        <Typography
                          variant="body2"
                          sx={{ fontWeight: "medium" }}
                        >
                          <TruncatedText limit={12} truncateMiddle>
                            {maker.peer_id}
                          </TruncatedText>
                        </Typography>
                        <Typography variant="caption" color="textSecondary">
                          v{maker.version}
                        </Typography>
                      </Box>
                    </Box>

                    {/* Price */}
                    <Box sx={{ textAlign: "center", minWidth: 120 }}>
                      <Typography variant="body2" sx={{ fontWeight: "medium" }}>
                        <MoneroBitcoinExchangeRate
                          rate={satsToBtc(maker.quote.price)}
                        />
                      </Typography>
                      <Typography variant="caption" color="textSecondary">
                        <SatsAmount amount={maker.quote.min_quantity} /> -{" "}
                        <SatsAmount amount={maker.quote.max_quantity} />
                      </Typography>
                    </Box>

                    {/* Accept Button */}
                    <PromiseInvokeButton
                      variant="contained"
                      color="success"
                      size="large"
                      sx={{ minWidth: 100, fontWeight: "bold" }}
                      onInvoke={() => resolveApproval(request_id, true)}
                      displayErrorSnackbar
                    >
                      Accept
                    </PromiseInvokeButton>
                  </Box>

                  {index < pendingSelectMakerApprovals.length - 1 && (
                    <Divider sx={{ my: 1 }} />
                  )}
                </Box>
              );
            })}
          </Box>
        </Box>
      )}

      {pendingSelectMakerApprovals.length === 0 && (
        <Paper variant="outlined" sx={{ p: 3, textAlign: "center" }}>
          <Typography variant="body1" color="textSecondary">
            Searching for available makers...
          </Typography>
          <Typography variant="body2" color="textSecondary" sx={{ mt: 1 }}>
            Please wait while we find the best offers for your swap.
          </Typography>
        </Paper>
      )}
    </Box>
  );
}

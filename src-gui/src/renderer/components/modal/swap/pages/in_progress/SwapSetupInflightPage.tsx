import { useState, useEffect } from "react";
import { resolveApproval } from "renderer/rpc";
import {
  PendingLockBitcoinApprovalRequest,
  TauriSwapProgressEventContent,
} from "models/tauriModelExt";
import {
  SatsAmount,
  PiconeroAmount,
  MoneroBitcoinExchangeRateFromAmounts,
} from "renderer/components/other/Units";
import { Box, Typography, Divider } from "@mui/material";
import { useActiveSwapId, usePendingLockBitcoinApproval } from "store/hooks";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import CircularProgressWithSubtitle from "../../CircularProgressWithSubtitle";
import CheckIcon from "@mui/icons-material/Check";
import ArrowRightAltIcon from "@mui/icons-material/ArrowRightAlt";
import TruncatedText from "renderer/components/other/TruncatedText";

/// A hook that returns the LockBitcoin confirmation request for the active swap
/// Returns null if no confirmation request is found
function useActiveLockBitcoinApprovalRequest(): PendingLockBitcoinApprovalRequest | null {
  const approvals = usePendingLockBitcoinApproval();
  const activeSwapId = useActiveSwapId();

  return (
    approvals?.find(
      (r) => r.content.details.content.swap_id === activeSwapId,
    ) || null
  );
}

export default function SwapSetupInflightPage({
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

  // If we do not have an approval request yet for the Bitcoin lock transaction, we haven't received the offer from Alice yet
  // Display a loading spinner to the user for as long as the swap_setup request is in flight
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

  const { btc_network_fee, monero_receive_pool, xmr_receive_amount } =
    request.content.details.content;

  return (
    <InfoBox
      title={null}
      icon={<></>}
      loading={false}
      mainContent={
        <>
          <Divider />
          <Box
            sx={{
              display: "flex",
              flexDirection: "row",
              gap: 4,
              marginBlock: 2,
              flex: 1,
              justifyContent: "space-around",
            }}
          >
            {/* Input Section */}
            <Box
              sx={{
                display: "flex",
                flexDirection: "column",
                gap: 1,
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Box
                sx={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 1,
                }}
              >
                <Box
                  sx={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: 1.5,
                    border: 1,
                    gap: "0.5rem 1rem",
                    borderColor: "warning.main",
                    borderRadius: 1,
                    backgroundColor: (theme) =>
                      theme.palette.warning.light + "10",
                    background: (theme) =>
                      `linear-gradient(135deg, ${theme.palette.warning.light}20, ${theme.palette.warning.light}05)`,
                  }}
                >
                  <Typography
                    variant="h6"
                    sx={(theme) => ({
                      color: theme.palette.text.primary,
                      fontWeight: 700,
                      letterSpacing: 0.5,
                    })}
                  >
                    You send
                  </Typography>
                  <Typography
                    variant="h5"
                    sx={(theme) => ({
                      fontWeight: "bold",
                      color: theme.palette.warning.dark,
                      textShadow: "0 1px 2px rgba(0,0,0,0.1)",
                    })}
                  >
                    <SatsAmount amount={btc_lock_amount} />
                  </Typography>
                </Box>

                <Box
                  sx={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: 1.5,
                    borderLeft: 3,
                    borderColor: "warning.light",
                    marginLeft: 2,
                    backgroundColor: (theme) => theme.palette.action.hover,
                    borderRadius: "0 8px 8px 0",
                    boxShadow: "0 2px 8px rgba(0,0,0,0.05)",
                  }}
                >
                  <Typography
                    variant="caption"
                    sx={(theme) => ({ color: theme.palette.text.secondary })}
                  >
                    Network fees
                  </Typography>
                  <Typography variant="caption">
                    <SatsAmount amount={btc_network_fee} />
                  </Typography>
                </Box>
              </Box>
            </Box>

            {/* Pulsating Arrow */}
            <Box
              sx={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <ArrowRightAltIcon
                sx={{
                  fontSize: "3rem",
                  color: (theme) => theme.palette.primary.main,
                  animation: "slideArrow 2s infinite",
                  "@keyframes slideArrow": {
                    "0%": {
                      opacity: 0.6,
                      transform: "translateX(-8px)",
                    },
                    "50%": {
                      opacity: 1,
                      transform: "translateX(8px)",
                    },
                    "100%": {
                      opacity: 0.6,
                      transform: "translateX(-8px)",
                    },
                  },
                }}
              />
            </Box>

            {/* Output Section */}
            <Box
              sx={{
                display: "flex",
                flexDirection: "column",
                gap: 1,
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              {monero_receive_pool.length === 1 ? (
                <Box
                  sx={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: 2.5,
                    border: 2,
                    borderColor: "success.main",
                    borderRadius: 2,
                    backgroundColor: (theme) =>
                      theme.palette.success.light + "15",
                    boxShadow: "0 4px 12px rgba(0,0,0,0.1)",
                    background: (theme) =>
                      `linear-gradient(135deg, ${theme.palette.success.light}20, ${theme.palette.success.light}05)`,
                  }}
                >
                  <Typography
                    variant="h6"
                    sx={(theme) => ({
                      color: theme.palette.text.primary,
                      fontWeight: 700,
                      letterSpacing: 0.5,
                    })}
                  >
                    You receive
                  </Typography>
                  <Typography
                    variant="h5"
                    sx={(theme) => ({
                      fontWeight: "bold",
                      color: theme.palette.success.dark,
                      textShadow: "0 1px 2px rgba(0,0,0,0.1)",
                    })}
                  >
                    <PiconeroAmount amount={xmr_receive_amount} />
                  </Typography>
                </Box>
              ) : (
                <Box sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
                  {monero_receive_pool.map((pool) => (
                    <Box
                      key={pool.address}
                      sx={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        padding: 1.5,
                        border: 1,
                        borderColor: "success.main",
                        borderRadius: 1,
                        backgroundColor: (theme) =>
                          theme.palette.success.light + "10",
                      }}
                    >
                      <Box
                        sx={{
                          display: "flex",
                          flexDirection: "column",
                          gap: 0.5,
                        }}
                      >
                        <Typography
                          variant="body2"
                          sx={(theme) => ({
                            color: theme.palette.text.primary,
                            fontSize: "0.875rem",
                            fontWeight: 600,
                          })}
                        >
                          {pool.label === "user address"
                            ? "Your Wallet"
                            : pool.label}
                        </Typography>
                        <Typography
                          variant="body2"
                          sx={{
                            fontFamily: "monospace",
                            fontSize: "0.75rem",
                            color: (theme) => theme.palette.text.secondary,
                          }}
                        >
                          <TruncatedText truncateMiddle limit={20}>
                            {pool.address}
                          </TruncatedText>
                        </Typography>
                      </Box>
                      <Box
                        sx={{
                          display: "flex",
                          flexDirection: "column",
                          alignItems: "flex-end",
                          gap: 0.5,
                        }}
                      >
                        <Typography
                          variant="body2"
                          sx={(theme) => ({
                            fontWeight: "bold",
                            color: theme.palette.success.main,
                            fontSize: "0.875rem",
                          })}
                        >
                          <PiconeroAmount
                            amount={
                              (pool.percentage * xmr_receive_amount) / 100
                            }
                          />
                        </Typography>
                        <Typography
                          variant="caption"
                          sx={(theme) => ({
                            color: theme.palette.text.secondary,
                          })}
                        >
                          {pool.percentage}%
                        </Typography>
                      </Box>
                    </Box>
                  ))}
                </Box>
              )}
            </Box>
          </Box>
        </>
      }
      additionalContent={
        <Box
          sx={{
            marginTop: 2,
            display: "flex",
            justifyContent: "center",
            gap: 2,
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

          <PromiseInvokeButton
            variant="contained"
            color="primary"
            size="large"
            onInvoke={() => resolveApproval(request.content.request_id, true)}
            displayErrorSnackbar
            requiresContext
            endIcon={<CheckIcon />}
          >
            {`Confirm (${timeLeft}s)`}
          </PromiseInvokeButton>
        </Box>
      }
    />
  );
}

/**
 * Pure presentational components -------------------------------------------------
 * They live in the same file to avoid additional imports yet keep
 * JSX for the main page tidy. All styling values are kept identical
 * to their previous inline counterparts so that the visual appearance
 * stays exactly the same while making the code easier to reason about.
 */

interface BitcoinSendSectionProps {
  btc_lock_amount: number;
  btc_network_fee: number;
}

const BitcoinSendSection = ({
  btc_lock_amount,
  btc_network_fee,
}: BitcoinSendSectionProps) => (
  <Box
    sx={{
      display: "flex",
      flexDirection: "column",
      gap: 1,
      alignItems: "center",
      justifyContent: "center",
    }}
  >
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        gap: 1,
      }}
    >
      {/* Main send summary */}
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          padding: 1.5,
          border: 1,
          gap: "0.5rem 1rem",
          borderColor: "warning.main",
          borderRadius: 1,
          backgroundColor: (theme) => theme.palette.warning.light + "10",
          background: (theme) =>
            `linear-gradient(135deg, ${theme.palette.warning.light}20, ${theme.palette.warning.light}05)`,
        }}
      >
        <Typography
          variant="h6"
          sx={(theme) => ({
            color: theme.palette.text.primary,
            fontWeight: 700,
            letterSpacing: 0.5,
          })}
        >
          You send
        </Typography>
        <Typography
          variant="h5"
          sx={(theme) => ({
            fontWeight: "bold",
            color: theme.palette.warning.dark,
            textShadow: "0 1px 2px rgba(0,0,0,0.1)",
          })}
        >
          <SatsAmount amount={btc_lock_amount} />
        </Typography>
      </Box>

      {/* Network fee */}
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          padding: 1.5,
          borderLeft: 3,
          borderColor: "warning.light",
          marginLeft: 2,
          backgroundColor: (theme) => theme.palette.action.hover,
          borderRadius: "0 8px 8px 0",
          boxShadow: "0 2px 8px rgba(0,0,0,0.05)",
        }}
      >
        <Typography
          variant="caption"
          sx={(theme) => ({ color: theme.palette.text.secondary })}
        >
          Network fees
        </Typography>
        <Typography variant="caption">
          <SatsAmount amount={btc_network_fee} />
        </Typography>
      </Box>
    </Box>
  </Box>
);

interface PoolBreakdownProps {
  monero_receive_pool: Array<{
    address: string;
    label: string;
    percentage: number;
  }>;
  xmr_receive_amount: number;
}

const PoolBreakdown = ({
  monero_receive_pool,
  xmr_receive_amount,
}: PoolBreakdownProps) => (
  <Box sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
    {monero_receive_pool.map((pool) => (
      <Box
        key={pool.address}
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          padding: 1.5,
          border: 1,
          borderColor: "success.main",
          borderRadius: 1,
          backgroundColor: (theme) => theme.palette.success.light + "10",
        }}
      >
        <Box sx={{ display: "flex", flexDirection: "column", gap: 0.5 }}>
          <Typography
            variant="body2"
            sx={(theme) => ({
              color: theme.palette.text.primary,
              fontSize: "0.875rem",
              fontWeight: 600,
            })}
          >
            {pool.label === "user address" ? "Your Wallet" : pool.label}
          </Typography>
          <Typography
            variant="body2"
            sx={{
              fontFamily: "monospace",
              fontSize: "0.75rem",
              color: (theme) => theme.palette.text.secondary,
            }}
          >
            <TruncatedText truncateMiddle limit={20}>
              {pool.address}
            </TruncatedText>
          </Typography>
        </Box>
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            alignItems: "flex-end",
            gap: 0.5,
          }}
        >
          <Typography
            variant="body2"
            sx={(theme) => ({
              fontWeight: "bold",
              color: theme.palette.success.main,
              fontSize: "0.875rem",
            })}
          >
            <PiconeroAmount
              amount={(pool.percentage * Number(xmr_receive_amount)) / 100}
            />
          </Typography>
          <Typography
            variant="caption"
            sx={(theme) => ({ color: theme.palette.text.secondary })}
          >
            {pool.percentage}%
          </Typography>
        </Box>
      </Box>
    ))}
  </Box>
);

interface MoneroReceiveSectionProps {
  monero_receive_pool: PoolBreakdownProps["monero_receive_pool"];
  xmr_receive_amount: number;
}

const MoneroReceiveSection = ({
  monero_receive_pool,
  xmr_receive_amount,
}: MoneroReceiveSectionProps) => (
  <Box
    sx={{
      display: "flex",
      flexDirection: "column",
      gap: 1,
      alignItems: "center",
      justifyContent: "center",
    }}
  >
    {monero_receive_pool.length === 1 ? (
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          padding: 2.5,
          border: 2,
          borderColor: "success.main",
          borderRadius: 2,
          backgroundColor: (theme) => theme.palette.success.light + "15",
          boxShadow: "0 4px 12px rgba(0,0,0,0.1)",
          background: (theme) =>
            `linear-gradient(135deg, ${theme.palette.success.light}20, ${theme.palette.success.light}05)`,
        }}
      >
        <Typography
          variant="h6"
          sx={(theme) => ({
            color: theme.palette.text.primary,
            fontWeight: 700,
            letterSpacing: 0.5,
          })}
        >
          You receive
        </Typography>
        <Typography
          variant="h5"
          sx={(theme) => ({
            fontWeight: "bold",
            color: theme.palette.success.dark,
            textShadow: "0 1px 2px rgba(0,0,0,0.1)",
          })}
        >
          <PiconeroAmount amount={xmr_receive_amount} />
        </Typography>
      </Box>
    ) : (
      <PoolBreakdown
        monero_receive_pool={monero_receive_pool}
        xmr_receive_amount={xmr_receive_amount}
      />
    )}
  </Box>
);

// Arrow animation styling extracted for reuse
const arrowSx = {
  fontSize: "3rem",
  color: (theme: any) => theme.palette.primary.main,
  animation: "slideArrow 2s infinite",
  "@keyframes slideArrow": {
    "0%": {
      opacity: 0.6,
      transform: "translateX(-8px)",
    },
    "50%": {
      opacity: 1,
      transform: "translateX(8px)",
    },
    "100%": {
      opacity: 0.6,
      transform: "translateX(-8px)",
    },
  },
} as const;

const AnimatedArrow = () => (
  <Box
    sx={{ display: "flex", alignItems: "center", justifyContent: "center" }}
  >
    <ArrowRightAltIcon sx={arrowSx} />
  </Box>
);
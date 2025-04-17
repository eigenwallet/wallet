import { useState, useEffect } from 'react';
import { resolveConfirmation } from 'renderer/rpc';
import { TauriSwapProgressEventContent } from 'models/tauriModelExt';
import {
  SatsAmount,
  PiconeroAmount,
  MoneroBitcoinExchangeRateFromAmounts
} from 'renderer/components/other/Units';
import {
  Box,
  Typography,
  Divider,
} from '@material-ui/core';
import { makeStyles, createStyles, Theme } from '@material-ui/core/styles';
import { useAppSelector } from 'store/hooks';
import PromiseInvokeButton from 'renderer/components/PromiseInvokeButton';
import InfoBox from 'renderer/components/modal/swap/InfoBox';
import CircularProgressWithSubtitle from '../../CircularProgressWithSubtitle';
import { ConfirmationEvent } from 'models/tauriModel';

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    paper: {
      width: '100%',
      padding: theme.spacing(3),
      borderRadius: theme.shape.borderRadius,
    },
    detailGrid: {
      display: 'grid',
      gridTemplateColumns: 'auto 1fr',
      rowGap: theme.spacing(1),
      columnGap: theme.spacing(2),
      alignItems: 'center',
      marginTop: theme.spacing(2),
      marginBottom: theme.spacing(2),
    },
    label: {
      color: theme.palette.text.secondary,
    },
    value: {
      fontWeight: theme.typography.fontWeightBold as unknown as any,
    },
    receiveValue: {
      fontWeight: theme.typography.fontWeightBold as unknown as any,
      color: theme.palette.success.main,
    },
    progressBar: {
      marginTop: theme.spacing(1),
      height: 6,
      borderRadius: 3,
    },
    actions: {
      marginTop: theme.spacing(2),
      display: 'flex',
      justifyContent: 'flex-end',
      gap: theme.spacing(2),
    },
    cancelButton: {
      color: theme.palette.text.secondary,
    },
  })
);

// Find the PreBtcLock confirmation
const findPreBtcLockRequest = (
  confirmations: Record<string, ConfirmationEvent>
): ConfirmationEvent | undefined => Object.values(confirmations)
  .find(r => r.content.details.type === 'PreBtcLock' && r.state === 'Pending');

export default function SwapSetupInflightPage({
  btc_lock_amount,
}: TauriSwapProgressEventContent<'SwapSetupInflight'>) {
  const classes = useStyles();
  const pending = useAppSelector(state => state.rpc.state.pendingConfirmations);
  const request = findPreBtcLockRequest(pending);

  const [timeLeft, setTimeLeft] = useState<number>(0);

  const expiresAtMs = request?.state === "Pending" ? request.content.expiration_ts * 1000 : 0;

  useEffect(() => {
    const tick = () => {
      const remainingMs = Math.max(expiresAtMs - Date.now(), 0);
      setTimeLeft(Math.ceil(remainingMs / 1000));
    };

    tick();
    const id = setInterval(tick, 250);
    return () => clearInterval(id);
  }, [expiresAtMs]);

  // If we do not have a confirmation request yet, we haven't received the offer yet
  // Display a loading spinner to the user
  // The spinner will be displayed as long as the swap_setup request is in flight
  if (!request) {
    return <CircularProgressWithSubtitle description={<>Negotiating offer for <SatsAmount amount={btc_lock_amount} /></>} />;
  }

  const { btc_network_fee, xmr_receive_amount } = request.content.details.content;

  return (
    <InfoBox
      title="Confirm Details"
      icon={<></>}
      loading={false}
      mainContent={
        <>
          <Divider />
          <Box className={classes.detailGrid}>
            <Typography className={classes.label}>You send</Typography>
            <Typography className={classes.value}>
              <SatsAmount amount={btc_lock_amount} />
            </Typography>

            <Typography className={classes.label}>You pay (Bitcoin network fees)</Typography>
            <Typography className={classes.value}>
              <SatsAmount amount={btc_network_fee} />
            </Typography>

            <Typography className={classes.label}>You receive</Typography>
            <Typography className={classes.receiveValue}>
              <PiconeroAmount amount={xmr_receive_amount} />
            </Typography>

            <Typography className={classes.label}>Exchange rate</Typography>
            <Typography className={classes.value}>
              <MoneroBitcoinExchangeRateFromAmounts
                satsAmount={btc_lock_amount}
                piconeroAmount={xmr_receive_amount}
                displayMarkup
              />
            </Typography>
          </Box>
        </>
      }
      additionalContent={
        <Box className={classes.actions}>
          <PromiseInvokeButton
            variant="text"
            size="large"
            className={classes.cancelButton}
            onInvoke={() => resolveConfirmation(request.content.request_id, false)}
            displayErrorSnackbar
            requiresContext
          >
            Deny
          </PromiseInvokeButton>

          <PromiseInvokeButton
            variant="contained"
            color="primary"
            size="large"
            onInvoke={() => resolveConfirmation(request.content.request_id, true)}
            displayErrorSnackbar
            requiresContext
          >
            {`Confirm & lock BTC (${timeLeft}s)`}
          </PromiseInvokeButton>
        </Box>
      }
    />
  );
}
import {
  Box,
  Typography,
  makeStyles,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
} from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { useState } from "react";
import { getWalletDescriptor } from "renderer/rpc";
import { ExportBitcoinWalletResponse } from "models/tauriModel";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import ActionableMonospaceTextBox from "renderer/components/other/ActionableMonospaceTextBox";

const useStyles = makeStyles((theme) => ({
  content: {
    display: "flex",
    flexDirection: "column",
    alignItems: "flex-start",
    gap: theme.spacing(2),
  }
}));

export default function ExportDataBox() {
  const classes = useStyles();
  const [walletDescriptor, setWalletDescriptor] = useState<ExportBitcoinWalletResponse | null>(null);

  const handleSuccess = (value: ExportBitcoinWalletResponse) => {
    setWalletDescriptor(value) 
  }

  const handleCloseDialog = () => {
    setWalletDescriptor(null);
  };

  const parseWalletDescriptor = (walletDescriptor: ExportBitcoinWalletResponse) => {
    const descriptor = JSON.parse(walletDescriptor.wallet_descriptor.descriptor);
    return descriptor;
  }

  return (
    <InfoBox
      title="Export Data"
      icon={null}
      loading={false}
      mainContent={
        <Box className={classes.content}>
          <Typography variant="body1">
            Export your Bitcoin wallet descriptor for backup or recovery purposes.
            The wallet descriptor is a JSON object that can be used to derive the wallet's private keys.
            It can be imported into other Bitcoin wallets or services that support the descriptor format.
            It should thus be stored securely.
          </Typography>
          <PromiseInvokeButton
            variant="outlined"
            onInvoke={getWalletDescriptor}
            onSuccess={handleSuccess}
            displayErrorSnackbar={true}
          >
            Export Bitcoin Wallet Descriptor
          </PromiseInvokeButton>
        </Box>
      }
      additionalContent={
        <Dialog open={walletDescriptor !== null} onClose={handleCloseDialog} maxWidth="md" fullWidth>
          <DialogTitle>Bitcoin Wallet Descriptor</DialogTitle>
          <DialogContent>
            {walletDescriptor && (
              <ActionableMonospaceTextBox
                content={JSON.stringify(parseWalletDescriptor(walletDescriptor), null, 4)}
                displayCopyIcon={true}
                enableQrCode={false}
              />
            )}
          </DialogContent>
          <DialogActions>
            <Button onClick={handleCloseDialog} color="primary">
              Close
            </Button>
          </DialogActions>
        </Dialog>
      }
    />
  );
}



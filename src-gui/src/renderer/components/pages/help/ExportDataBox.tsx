import {
  Box,
  Typography,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Link,
  DialogContentText,
  Alert,
} from "@mui/material";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { useState } from "react";
import { getWalletDescriptor, exportSeed } from "renderer/rpc";
import { ExportBitcoinWalletResponse, ExportSeedResponse } from "models/tauriModel";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import ActionableMonospaceTextBox from "renderer/components/other/ActionableMonospaceTextBox";

export default function ExportDataBox() {
  const [walletDescriptor, setWalletDescriptor] =
    useState<ExportBitcoinWalletResponse | null>(null);
  const [seedData, setSeedData] = useState<ExportSeedResponse | null>(null);

  const handleCloseWalletDialog = () => {
    setWalletDescriptor(null);
  };

  const handleCloseSeedDialog = () => {
    setSeedData(null);
  };

  return (
    <InfoBox
      title="Export Wallet Data"
      icon={null}
      loading={false}
      mainContent={
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            alignItems: "flex-start",
            gap: 2,
          }}
        >
          <Typography variant="subtitle2">
            You can export your wallet data for backup or recovery purposes. 
            This includes both the Bitcoin wallet descriptor and your master seed.
            Please make sure to store them securely.
          </Typography>
        </Box>
      }
      additionalContent={
        <>
          <PromiseInvokeButton
            variant="outlined"
            onInvoke={getWalletDescriptor}
            onSuccess={setWalletDescriptor}
            displayErrorSnackbar={true}
          >
            Export Bitcoin Wallet Descriptor
          </PromiseInvokeButton>
          
          <PromiseInvokeButton
            variant="outlined"
            onInvoke={exportSeed}
            onSuccess={setSeedData}
            displayErrorSnackbar={true}
          >
            Export Master Seed
          </PromiseInvokeButton>

          {walletDescriptor !== null && (
            <WalletDescriptorModal
              open={walletDescriptor !== null}
              onClose={handleCloseWalletDialog}
              walletDescriptor={walletDescriptor}
            />
          )}

          {seedData !== null && (
            <SeedExportModal
              open={seedData !== null}
              onClose={handleCloseSeedDialog}
              seedData={seedData}
            />
          )}
        </>
      }
    />
  );
}

function WalletDescriptorModal({
  open,
  onClose,
  walletDescriptor,
}: {
  open: boolean;
  onClose: () => void;
  walletDescriptor: ExportBitcoinWalletResponse;
}) {
  const parsedDescriptor = JSON.parse(
    walletDescriptor.wallet_descriptor["descriptor"],
  );
  const stringifiedDescriptor = JSON.stringify(parsedDescriptor, null, 4);

  const exportToFile = () => {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const filename = `bitcoin-wallet-descriptor-${timestamp}.json`;
    
    const blob = new Blob([stringifiedDescriptor], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>Bitcoin Wallet Descriptor</DialogTitle>
      <DialogContent>
        <DialogContentText>
          <ul style={{ marginTop: 0 }}>
            <li>
              The text below contains the wallet descriptor of the internal
              Bitcoin wallet. It contains your private key and can be used to
              derive your wallet. It should thus be stored securely.
            </li>
            <li>
              It can be imported into other Bitcoin wallets or services that
              support the descriptor format.
            </li>
            <li>
              For more information on what to do with the descriptor, see our{" "}
              <Link
                href="https://github.com/UnstoppableSwap/core/blob/master/dev-docs/asb/README.md#exporting-the-bitcoin-wallet-descriptor"
                target="_blank"
              >
                documentation
              </Link>
            </li>
          </ul>
        </DialogContentText>
        <ActionableMonospaceTextBox
          content={stringifiedDescriptor}
          displayCopyIcon={true}
          enableQrCode={false}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={exportToFile} color="secondary" variant="outlined">
          Save to File
        </Button>
        <Button onClick={onClose} color="primary" variant="contained">
          Done
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function SeedExportModal({
  open,
  onClose,
  seedData,
}: {
  open: boolean;
  onClose: () => void;
  seedData: ExportSeedResponse;
}) {
  const exportToFile = (content: string, filename: string, type: string = 'text/plain') => {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  const exportPolyseedMnemonic = () => {
    if (seedData.polyseed_mnemonic) {
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const filename = `polyseed-mnemonic-${timestamp}.txt`;
      let content = seedData.polyseed_mnemonic;
      if (seedData.polyseed_warning) {
        content = `${seedData.polyseed_warning}\n\n${content}`;
      }
      exportToFile(content, filename);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>Master Seed Export</DialogTitle>
      <DialogContent>
        <DialogContentText>
          Your master seed can be exported as a polyseed mnemonic:
        </DialogContentText>

        {/* Polyseed Section */}
        {seedData.polyseed_available && seedData.polyseed_mnemonic ? (
          <Box sx={{ mt: 2, mb: 3 }}>
            <Typography variant="h6" gutterBottom>
              Polyseed Mnemonic
            </Typography>
            {seedData.polyseed_warning && (
              <Alert severity="warning" sx={{ mb: 2 }}>
                {seedData.polyseed_warning}
              </Alert>
            )}
            <ActionableMonospaceTextBox
              content={seedData.polyseed_mnemonic}
              displayCopyIcon={true}
              enableQrCode={false}
            />
            <Box sx={{ mt: 1 }}>
              <Button onClick={exportPolyseedMnemonic} size="small" variant="outlined">
                Save to File
              </Button>
            </Box>
          </Box>
        ) : (
          <Box sx={{ mt: 2, mb: 3 }}>
            <Typography variant="h6" gutterBottom>
              Polyseed Mnemonic
            </Typography>
            <Alert severity="info">
              Polyseed format is not available for this seed.
            </Alert>
          </Box>
        )}

        {/* Note Section */}
        {seedData.note && (
          <Alert severity="info" sx={{ mt: 2 }}>
            <Typography variant="body2">
              <strong>Note:</strong> {seedData.note}
            </Typography>
          </Alert>
        )}

        <Alert severity="error" sx={{ mt: 2 }}>
          <Typography variant="body2">
            <strong>Security Warning:</strong> Never share your seed with anyone. 
            Anyone with access to your seed can control your funds. Store it securely offline.
          </Typography>
        </Alert>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} color="primary" variant="contained">
          Done
        </Button>
      </DialogActions>
    </Dialog>
  );
}

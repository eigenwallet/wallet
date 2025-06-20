import {
  Box,
  Typography,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  FormControl,
  FormLabel,
  RadioGroup,
  FormControlLabel,
  Radio,
  Alert,
  Paper,
} from "@mui/material";
import { useState, useEffect } from "react";
import { respondToSeedInitialization } from "renderer/rpc";

interface SeedInitializationModalProps {
  open: boolean;
  requestId: string;
  onResponse: () => void;
}

export default function SeedInitializationModal({
  open,
  requestId,
  onResponse,
}: SeedInitializationModalProps) {
  const [choice, setChoice] = useState<"random" | "recover">("random");
  const [mnemonic, setMnemonic] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async () => {
    if (choice === "recover" && !mnemonic.trim()) {
      setError("Please enter your polyseed mnemonic");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const responseChoice = choice === "random" 
        ? "random" 
        : { type: "recover" as const, mnemonic: mnemonic.trim() };
      
      await respondToSeedInitialization(requestId, responseChoice);
      onResponse();
    } catch (err) {
      setError(`Failed to submit choice: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setMnemonic("");
    setError(null);
    setChoice("random");
  };

  return (
    <Dialog
      open={open}
      maxWidth="md"
      fullWidth
      disableEscapeKeyDown
      onClose={() => {}} // Prevent closing by clicking outside
    >
      <DialogTitle>
        <Typography variant="h5" component="h2">
          Initialize Your Wallet
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          Choose how to set up your wallet seed
        </Typography>
      </DialogTitle>
      
      <DialogContent>
        <Box sx={{ mt: 2 }}>
          <FormControl component="fieldset" sx={{ mb: 3 }}>
            <FormLabel component="legend">Wallet Setup Option</FormLabel>
            <RadioGroup
              value={choice}
              onChange={(e) => setChoice(e.target.value as "random" | "recover")}
            >
              <FormControlLabel
                value="random"
                control={<Radio />}
                label="Create New Wallet (Random Seed)"
              />
              <FormControlLabel
                value="recover"
                control={<Radio />}
                label="Restore from Polyseed Mnemonic"
              />
            </RadioGroup>
          </FormControl>

          {choice === "random" && (
            <Alert severity="info" sx={{ mb: 2 }}>
              <Typography variant="body2">
                A new random seed will be generated for your wallet. Make sure to export and backup 
                your seed after the wallet is created.
              </Typography>
            </Alert>
          )}

          {choice === "recover" && (
            <>
              <Alert severity="warning" sx={{ mb: 2 }}>
                <Typography variant="body2">
                  <strong>Note:</strong> Polyseed format contains only 150 bits of your original 256-bit seed. 
                  This software can reconstruct the full seed deterministically, but make sure you're using 
                  the same UnstoppableSwap software that created the polyseed.
                </Typography>
              </Alert>

              <TextField
                fullWidth
                multiline
                rows={4}
                label="Polyseed Mnemonic"
                placeholder="word1 word2 word3 ... (16 words separated by spaces)"
                value={mnemonic}
                onChange={(e) => setMnemonic(e.target.value)}
                variant="outlined"
                sx={{ mb: 2 }}
                helperText="Enter the 16-word polyseed mnemonic separated by spaces"
              />
            </>
          )}

          {error && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {error}
            </Alert>
          )}

          <Paper elevation={1} sx={{ p: 2, bgcolor: "background.default" }}>
            <Typography variant="body2" color="text.secondary">
              <strong>Security Notice:</strong> Make sure you're in a secure environment when entering your seed. 
              Never share your seed with anyone as it provides full access to your wallet.
            </Typography>
          </Paper>
        </Box>
      </DialogContent>

      <DialogActions sx={{ p: 3, pt: 1 }}>
        <Button
          onClick={resetForm}
          variant="outlined"
          disabled={loading}
        >
          Reset
        </Button>
        <Box sx={{ flex: 1 }} />
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={loading || (choice === "recover" && !mnemonic.trim())}
        >
          {loading ? "Processing..." : choice === "random" ? "Create New Wallet" : "Restore Wallet"}
        </Button>
      </DialogActions>
    </Dialog>
  );
} 
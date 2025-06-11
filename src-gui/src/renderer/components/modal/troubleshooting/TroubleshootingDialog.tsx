import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Box,
  Chip,
} from "@mui/material";
import {
  NetworkCheck,
  Router,
  Security,
  Update,
  People,
  Warning,
  Help,
} from "@mui/icons-material";

interface TroubleshootingDialogProps {
  open: boolean;
  onClose: () => void;
  errorCategory: string;
  errorMessage: string;
  totalAttempts: number;
  elapsedTime: number;
}

export default function TroubleshootingDialog({
  open,
  onClose,
  errorCategory,
  errorMessage,
  totalAttempts,
  elapsedTime,
}: TroubleshootingDialogProps) {
  const getSuggestionsForCategory = (category: string): Array<{ text: string; icon: JSX.Element }> => {
    switch (category) {
      case "Network":
        return [
          { text: "Check your internet connection", icon: <NetworkCheck /> },
          { text: "Verify DNS settings", icon: <Router /> },
          { text: "Try connecting to a different network", icon: <NetworkCheck /> },
          { text: "Disable VPN temporarily if enabled", icon: <Security /> },
          { text: "Check firewall settings", icon: <Security /> },
        ];
      case "Timeout":
        return [
          { text: "Check if the remote service is running", icon: <Help /> },
          { text: "Try again in a few minutes", icon: <Warning /> },
          { text: "Consider using a different endpoint", icon: <Router /> },
          { text: "Check for network congestion", icon: <NetworkCheck /> },
        ];
      case "Auth":
        return [
          { text: "Check your credentials", icon: <Security /> },
          { text: "Verify your account permissions", icon: <People /> },
          { text: "Ensure your account is not locked", icon: <Security /> },
        ];
      case "Protocol":
        return [
          { text: "Update the application to the latest version", icon: <Update /> },
          { text: "Check for compatibility issues", icon: <Warning /> },
          { text: "Restart the application", icon: <Help /> },
        ];
      case "PeerUnavailable":
        return [
          { text: "The peer may be offline or unreachable", icon: <People /> },
          { text: "Try connecting to a different peer", icon: <Router /> },
          { text: "Check the peer's address for typos", icon: <Help /> },
          { text: "Verify the peer is on the same network", icon: <NetworkCheck /> },
        ];
      case "Resource":
        return [
          { text: "Wait a moment and try again", icon: <Warning /> },
          { text: "Close other network-intensive applications", icon: <Help /> },
          { text: "Check system resources (CPU, memory)", icon: <Warning /> },
        ];
      default:
        return [
          { text: "Check application logs for more details", icon: <Help /> },
          { text: "Try restarting the application", icon: <Update /> },
          { text: "Ensure your system meets requirements", icon: <Warning /> },
          { text: "Contact support if the issue persists", icon: <People /> },
        ];
    }
  };

  const suggestions = getSuggestionsForCategory(errorCategory);

  const formatElapsedTime = (seconds: number): string => {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  const getSeverityColor = () => {
    if (totalAttempts >= 15) return "error";
    if (totalAttempts >= 10) return "warning";
    return "info";
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>Connection Troubleshooting</DialogTitle>
      
      <DialogContent>
        <Box sx={{ mb: 3 }}>
          <Typography variant="h6" gutterBottom>
            Problem Summary
          </Typography>
          
          <Box sx={{ display: "flex", flexWrap: "wrap", gap: 1, mb: 2 }}>
            <Chip 
              label={`Error Type: ${errorCategory}`} 
              color={getSeverityColor() as any}
              variant="outlined"
            />
            <Chip 
              label={`${totalAttempts} failed attempts`} 
              color={getSeverityColor() as any}
              variant="outlined"
            />
            <Chip 
              label={`Running for ${formatElapsedTime(elapsedTime)}`} 
              variant="outlined"
            />
          </Box>

          {errorMessage && (
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              <strong>Last Error:</strong> {errorMessage}
            </Typography>
          )}
        </Box>

        <Typography variant="h6" gutterBottom>
          Suggested Solutions
        </Typography>
        
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Try these solutions in order. Most connection issues can be resolved with the first few steps.
        </Typography>

        <List>
          {suggestions.map((suggestion, index) => (
            <ListItem key={index}>
              <ListItemIcon>{suggestion.icon}</ListItemIcon>
              <ListItemText 
                primary={suggestion.text}
                primaryTypographyProps={{ variant: "body2" }}
              />
            </ListItem>
          ))}
        </List>

        {totalAttempts >= 15 && (
          <Box sx={{ mt: 3, p: 2, backgroundColor: "error.light", borderRadius: 1 }}>
            <Typography variant="body2" color="error.contrastText">
              <strong>Persistent Connection Issues Detected</strong>
              <br />
              After {totalAttempts} failed attempts over {formatElapsedTime(elapsedTime)}, 
              there may be a more serious connectivity issue. Consider checking your 
              network configuration or contacting technical support.
            </Typography>
          </Box>
        )}
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose} color="primary">
          Close
        </Button>
      </DialogActions>
    </Dialog>
  );
}
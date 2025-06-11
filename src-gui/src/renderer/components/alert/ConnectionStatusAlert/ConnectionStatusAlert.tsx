import { Alert, AlertTitle, Box, Chip, LinearProgress, Typography } from "@mui/material";
import { useEffect, useState } from "react";
import ErrorIcon from "@mui/icons-material/Error";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ScheduleIcon from "@mui/icons-material/Schedule";
import SyncIcon from "@mui/icons-material/Sync";

interface ConnectionProgress {
  current_attempt: number;
  total_attempts: number;
  retries_left?: number | null;
  last_error: string;
  error_category: string;
  next_retry_in?: number | null; // seconds
  elapsed_time: number; // seconds
  state: string;
  target: string;
}

interface ConnectionStatusAlertProps {
  progress: ConnectionProgress;
  onSuggestionsClick?: () => void;
}

export default function ConnectionStatusAlert({ 
  progress, 
  onSuggestionsClick 
}: ConnectionStatusAlertProps) {
  const [timeLeft, setTimeLeft] = useState<number>(progress.next_retry_in || 0);

  useEffect(() => {
    if (progress.next_retry_in && progress.next_retry_in > 0) {
      setTimeLeft(progress.next_retry_in);
      
      const interval = setInterval(() => {
        setTimeLeft((prev) => Math.max(0, prev - 1));
      }, 1000);

      return () => clearInterval(interval);
    }
  }, [progress.next_retry_in]);

  const formatElapsedTime = (seconds: number): string => {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  const getStateIcon = () => {
    switch (progress.state) {
      case "Connected":
        return <CheckCircleIcon color="success" />;
      case "Connecting":
        return <SyncIcon className="animate-spin" />;
      case "WaitingToRetry":
        return <ScheduleIcon color="warning" />;
      case "Failed":
        return <ErrorIcon color="error" />;
      default:
        return <SyncIcon />;
    }
  };

  const getSeverity = () => {
    switch (progress.state) {
      case "Connected":
        return "success";
      case "Failed":
        return "error";
      case "WaitingToRetry":
        return progress.total_attempts >= 10 ? "warning" : "info";
      default:
        return "info";
    }
  };

  const getErrorCategoryColor = (category: string) => {
    switch (category) {
      case "Timeout":
        return "warning";
      case "Network":
        return "error";
      case "PeerUnavailable":
        return "info";
      case "Auth":
        return "error";
      default:
        return "default";
    }
  };

  const formatMessage = (): string => {
    switch (progress.state) {
      case "Initial":
        return `Preparing to connect to ${progress.target}`;
      case "Connecting":
        return `Connecting to ${progress.target} (attempt ${progress.current_attempt})`;
      case "WaitingToRetry":
        const retryInfo = progress.retries_left
          ? `${progress.retries_left} retries left`
          : "unlimited retries";
        
        const timeInfo = timeLeft > 0 ? ` in ${timeLeft}s` : "";
        
        return `Trying to reconnect to ${progress.target} (Last Error: ${progress.error_category}, ${progress.total_attempts} times failed, ${retryInfo})${timeInfo}`;
      case "Connected":
        return `Connected to ${progress.target}`;
      case "Failed":
        return `Failed to connect to ${progress.target} after ${progress.total_attempts} attempts`;
      case "Reconnecting":
        return `Connection to ${progress.target} lost, attempting to reconnect`;
      default:
        return `Unknown connection state for ${progress.target}`;
    }
  };

  const showRetryProgress = progress.state === "WaitingToRetry" && progress.next_retry_in && progress.next_retry_in > 0;
  const retryProgress = showRetryProgress 
    ? ((progress.next_retry_in! - timeLeft) / progress.next_retry_in!) * 100 
    : 0;

  return (
    <Alert 
      severity={getSeverity() as any} 
      icon={getStateIcon()}
      sx={{ 
        '& .MuiAlert-message': { width: '100%' },
        '& .animate-spin': {
          animation: 'spin 1s linear infinite',
        },
        '@keyframes spin': {
          '0%': { transform: 'rotate(0deg)' },
          '100%': { transform: 'rotate(360deg)' },
        },
      }}
    >
      <AlertTitle>
        Connection Status - {progress.target}
      </AlertTitle>
      
      <Box sx={{ mb: 1 }}>
        <Typography variant="body2">
          {formatMessage()}
        </Typography>
      </Box>

      <Box sx={{ display: "flex", flexWrap: "wrap", gap: 1, mb: 1 }}>
        <Chip 
          size="small" 
          label={`State: ${progress.state}`} 
          color={progress.state === "Connected" ? "success" : "default"}
        />
        
        {progress.total_attempts > 0 && (
          <Chip 
            size="small" 
            label={`Attempts: ${progress.total_attempts}`} 
            color={progress.total_attempts >= 10 ? "warning" : "default"}
          />
        )}
        
        {progress.error_category !== "Unknown" && progress.last_error && (
          <Chip 
            size="small" 
            label={`Error: ${progress.error_category}`} 
            color={getErrorCategoryColor(progress.error_category) as any}
          />
        )}
        
        <Chip 
          size="small" 
          label={`Elapsed: ${formatElapsedTime(progress.elapsed_time)}`} 
        />
      </Box>

      {showRetryProgress && (
        <Box sx={{ mb: 1 }}>
          <Typography variant="caption" color="text.secondary">
            Next retry in {timeLeft}s
          </Typography>
          <LinearProgress 
            variant="determinate" 
            value={retryProgress} 
            sx={{ mt: 0.5, height: 4 }}
          />
        </Box>
      )}

      {progress.total_attempts >= 5 && onSuggestionsClick && (
        <Box sx={{ mt: 1 }}>
          <Typography 
            variant="caption" 
            color="primary" 
            sx={{ cursor: "pointer", textDecoration: "underline" }}
            onClick={onSuggestionsClick}
          >
            View troubleshooting suggestions
          </Typography>
        </Box>
      )}
    </Alert>
  );
}
import {
  Box,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Chip,
  LinearProgress,
  useTheme,
} from "@mui/material";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { ReliableNodeInfo } from "models/tauriModel";
import NetworkWifiIcon from "@mui/icons-material/NetworkWifi";
import { useAppSelector } from "store/hooks";

export default function MoneroPoolHealthBox() {
  const { poolStatus, isLoading } = useAppSelector((state) => ({
    poolStatus: state.pool.status,
    isLoading: state.pool.isLoading,
  }));
  const theme = useTheme();

  const formatLatency = (latencyMs?: number) => {
    if (latencyMs === undefined || latencyMs === null) return "N/A";
    return `${Math.round(latencyMs)}ms`;
  };

  const formatSuccessRate = (rate: number) => {
    return `${(rate * 100).toFixed(1)}%`;
  };

  const getHealthColor = (healthyCount: number, reliableCount: number) => {
    if (reliableCount === 0) return theme.palette.error.main;
    if (reliableCount < 3) return theme.palette.warning.main;
    return theme.palette.success.main;
  };

  const renderHealthSummary = () => {
    if (!poolStatus) return null;

    const totalChecks = poolStatus.successful_health_checks + poolStatus.unsuccessful_health_checks;
    const overallSuccessRate = totalChecks > 0 
      ? (poolStatus.successful_health_checks / totalChecks) * 100 
      : 0;

    return (
      <Box sx={{ display: "flex", gap: 2, flexWrap: "wrap" }}>
        <Chip
          label={`${poolStatus.healthy_node_count} Healthy`}
          color={poolStatus.healthy_node_count > 0 ? "success" : "error"}
          variant="outlined"
          size="small"
        />
        <Chip
          label={`${poolStatus.reliable_node_count} Reliable`}
          color={poolStatus.reliable_node_count > 0 ? "success" : "warning"}
          variant="outlined"
          size="small"
        />
        <Chip
          label={`${overallSuccessRate.toFixed(1)}% Success Rate`}
          color={overallSuccessRate > 80 ? "success" : overallSuccessRate > 60 ? "warning" : "error"}
          variant="outlined"
          size="small"
        />
      </Box>
    );
  };

  const renderTopNodes = () => {
    if (!poolStatus || poolStatus.top_reliable_nodes.length === 0) {
      return (
        <Typography variant="body2" color="text.secondary">
          No reliable nodes available
        </Typography>
      );
    }

    return (
      <TableContainer>
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell>Node URL</TableCell>
              <TableCell align="right">Success Rate</TableCell>
              <TableCell align="right">Avg Latency</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {poolStatus.top_reliable_nodes.map((node: ReliableNodeInfo, index: number) => (
              <TableRow key={index}>
                <TableCell>
                  <Typography variant="caption" sx={{ wordBreak: "break-all" }}>
                    {node.url}
                  </Typography>
                </TableCell>
                <TableCell align="right">
                  <Typography variant="caption">
                    {formatSuccessRate(node.success_rate)}
                  </Typography>
                </TableCell>
                <TableCell align="right">
                  <Typography variant="caption">
                    {formatLatency(node.avg_latency_ms)}
                  </Typography>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    );
  };

  return (
    <InfoBox
      title={
        <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
          <NetworkWifiIcon />
          Monero Pool Health
        </Box>
      }
      mainContent={
        <Typography variant="subtitle2">
          Real-time health monitoring of the Monero node pool. Shows node availability, 
          success rates, and performance metrics.
        </Typography>
      }
      additionalContent={
        <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
          {poolStatus && renderHealthSummary()}
          
          {poolStatus && (
            <Box>
              <Typography variant="body2" sx={{ mb: 1, fontWeight: "medium" }}>
                Health Check Statistics
              </Typography>
              <Box sx={{ display: "flex", gap: 2, flexWrap: "wrap" }}>
                <Typography variant="caption" color="text.secondary">
                  Successful: {poolStatus.successful_health_checks.toLocaleString()}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  Failed: {poolStatus.unsuccessful_health_checks.toLocaleString()}
                </Typography>
              </Box>
            </Box>
          )}

          <Box>
            <Typography variant="body2" sx={{ mb: 1, fontWeight: "medium" }}>
              Top Reliable Nodes
            </Typography>
            {renderTopNodes()}
          </Box>
        </Box>
      }
      icon={null}
      loading={isLoading}
    />
  );
}
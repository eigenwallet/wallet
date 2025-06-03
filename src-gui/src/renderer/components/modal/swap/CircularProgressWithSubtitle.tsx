import { Box, CircularProgress, LinearProgress, Typography } from "@mui/material";
import makeStyles from '@mui/styles/makeStyles';
import { ReactNode } from "react";

const useStyles = makeStyles((theme) => ({
  subtitle: {
    paddingTop: theme.spacing(1),
  },
}));

export default function CircularProgressWithSubtitle({
  description,
}: {
  description: string | ReactNode;
}) {
  const classes = useStyles();

  return (
    <Box
      sx={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        flexDirection: "column"
      }}>
      <CircularProgress size={50} />
      <Typography variant="subtitle2" className={classes.subtitle}>
        {description}
      </Typography>
    </Box>
  );
}

export function LinearProgressWithSubtitle({
  description,
  value,
}: {
  description: string | ReactNode;
  value: number;
}) {
  const classes = useStyles();

  return (
    <Box
      style={{ gap: "0.5rem" }}
      sx={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center"
      }}>
      <Typography variant="subtitle2" className={classes.subtitle}>
        {description}
      </Typography>
      <Box sx={{
        width: "10rem"
      }}>
        <LinearProgress variant="determinate" value={value} />
      </Box>
    </Box>
  );
}
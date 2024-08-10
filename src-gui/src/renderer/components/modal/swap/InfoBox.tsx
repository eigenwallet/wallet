import { Box, LinearProgress, Paper, Typography } from "@mui/material";
import makeStyles from '@mui/styles/makeStyles';
import { ReactNode } from "react";

type Props = {
  title: ReactNode;
  mainContent: ReactNode;
  additionalContent: ReactNode;
  loading: boolean;
  icon: ReactNode;
};

const useStyles = makeStyles((theme) => ({
  outer: {
    padding: theme.spacing(1.5),
    overflow: "hidden",
    display: "flex",
    flexDirection: "column",
    gap: theme.spacing(1),
  },
  upperContent: {
    display: "flex",
    alignItems: "center",
    gap: theme.spacing(0.5),
  },
}));

export default function InfoBox({
  title,
  mainContent,
  additionalContent,
  icon,
  loading,
}: Props) {
  const classes = useStyles();

  return (
    <Paper variant="outlined" className={classes.outer}>
      <Typography variant="subtitle1">{title}</Typography>
      <Box className={classes.upperContent}>
        {icon}
        {mainContent}
      </Box>
      {loading ? <LinearProgress variant="indeterminate" /> : null}
      <Box>{additionalContent}</Box>
    </Paper>
  );
}

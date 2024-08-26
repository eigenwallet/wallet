import { adaptV4Theme, Box, CssBaseline } from "@mui/material";
import { indigo } from "@mui/material/colors";
import {
  createTheme,
  StyledEngineProvider,
  ThemeProvider,
} from "@mui/material/styles";
import makeStyles from "@mui/styles/makeStyles";
import { Route, MemoryRouter as Router, Routes } from "react-router-dom";
import Navigation, { drawerWidth } from "./navigation/Navigation";
import HelpPage from "./pages/help/HelpPage";
import HistoryPage from "./pages/history/HistoryPage";
import SwapPage from "./pages/swap/SwapPage";
import WalletPage from "./pages/wallet/WalletPage";
import GlobalSnackbarProvider from "./snackbar/GlobalSnackbarProvider";

const useStyles = makeStyles((theme) => ({
  innerContent: {
    padding: theme.spacing(4),
    marginLeft: drawerWidth,
    maxHeight: `100vh`,
    flex: 1,
  },
}));

const theme = createTheme(
  adaptV4Theme({
    palette: {
      mode: "dark",
      primary: {
        main: "#f4511e",
      },
      secondary: indigo,
      background: {
        default: "#303030",
        paper: "#424242",
      },
    },
    transitions: {
      create: () => "none",
    },
    props: {
      MuiButtonBase: {
        disableRipple: true,
      },
    },
  }),
);

function InnerContent() {
  const classes = useStyles();

  return (
    <Box className={classes.innerContent}>
      <Routes>
        <Route path="/swap" element={<SwapPage />} />
        <Route path="/history" element={<HistoryPage />} />
        <Route path="/wallet" element={<WalletPage />} />
        <Route path="/help" element={<HelpPage />} />
        <Route path="/" element={<SwapPage />} />
      </Routes>
    </Box>
  );
}

export default function App() {
  return (
    <StyledEngineProvider injectFirst>
      <ThemeProvider theme={theme}>
        <GlobalSnackbarProvider>
          <CssBaseline />
          <Router>
            <Navigation />
            <InnerContent />
          </Router>
        </GlobalSnackbarProvider>
      </ThemeProvider>
    </StyledEngineProvider>
  );
}

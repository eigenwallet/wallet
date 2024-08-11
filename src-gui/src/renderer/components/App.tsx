import { Box, CssBaseline, adaptV4Theme } from "@mui/material";
import makeStyles from "@mui/styles/makeStyles";
import {
  createTheme,
  ThemeProvider,
  Theme,
  StyledEngineProvider,
} from "@mui/material/styles";
import { indigo } from "@mui/material/colors";
import { MemoryRouter as Router, Routes, Route } from "react-router-dom";
import Navigation, { drawerWidth } from "./navigation/Navigation";
import HistoryPage from "./pages/history/HistoryPage";
import SwapPage from "./pages/swap/SwapPage";
import WalletPage from "./pages/wallet/WalletPage";
import HelpPage from "./pages/help/HelpPage";
import GlobalSnackbarProvider from "./snackbar/GlobalSnackbarProvider";

declare module "@mui/styles/defaultTheme" {
  // eslint-disable-next-line @typescript-eslint/no-empty-interface
  interface DefaultTheme extends Theme {}
}

const useStyles = makeStyles((theme) => ({
  innerContent: {
    padding: theme.spacing(4),
    marginLeft: drawerWidth,
    maxHeight: `100vh`,
    flex: 1,
  },
}));

const theme = createTheme({
  palette: {
    mode: "dark",
    primary: {
      main: "#f4511e",
    },
    secondary: indigo,
  },
  transitions: {
    create: () => "none",
  },
  components: {
    MuiButtonBase: {
      defaultProps: {
        disableRipple: true,
      },
    },
  },
});

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

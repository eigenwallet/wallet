import { Box, CssBaseline } from "@mui/material";
import {
  ThemeProvider,
  Theme,
  StyledEngineProvider,
} from "@mui/material/styles";
import "@tauri-apps/plugin-shell";
import { Route, MemoryRouter as Router, Routes } from "react-router-dom";
import Navigation, { drawerWidth } from "./navigation/Navigation";
import SettingsPage from "./pages/help/SettingsPage";
import HistoryPage from "./pages/history/HistoryPage";
import SwapPage from "./pages/swap/SwapPage";
import WalletPage from "./pages/wallet/WalletPage";
import GlobalSnackbarProvider from "./snackbar/GlobalSnackbarProvider";
import UpdaterDialog from "./modal/updater/UpdaterDialog";
import { useSettings } from "store/hooks";
import { Theme as ThemeEnum, themes } from "./theme";
import { useEffect, useState } from "react";
import { setupBackgroundTasks } from "renderer/background";
import "@fontsource/roboto";
import FeedbackPage from "./pages/feedback/FeedbackPage";
import IntroductionModal from "./modal/introduction/IntroductionModal";
import SeedInitializationModal from "./modal/SeedInitializationModal";
import { listen } from "@tauri-apps/api/event";

declare module "@mui/material/styles" {
  interface Theme {
    // Add your custom theme properties here if needed
  }
  interface ThemeOptions {
    // Add your custom theme options here if needed
  }
}

export default function App() {
  const [seedInitRequest, setSeedInitRequest] = useState<{
    requestId: string;
    open: boolean;
  } | null>(null);
  
  useEffect(() => {
    setupBackgroundTasks();
    
    // Listen for seed initialization requests
    const unlistenPromise = listen("seed-initialization-request", (event) => {
      const requestId = event.payload as string;
      setSeedInitRequest({ requestId, open: true });
    });

    return () => {
      unlistenPromise.then(fn => fn());
    };
  }, []);

  const theme = useSettings((s) => s.theme);
  const userHasSeenIntroduction = useSettings((s) => s.userHasSeenIntroduction);
  const currentTheme = themes[theme] || themes[ThemeEnum.Dark];

  console.log("Current theme:", { theme, currentTheme });

  const handleSeedInitResponse = () => {
    setSeedInitRequest(null);
  };

  const showIntroduction = !userHasSeenIntroduction && !seedInitRequest?.open;

  return (
    <StyledEngineProvider injectFirst>
      <ThemeProvider theme={currentTheme}>
        <CssBaseline />
        <GlobalSnackbarProvider>
          {seedInitRequest?.open && (
            <SeedInitializationModal
              open={seedInitRequest.open}
              requestId={seedInitRequest.requestId}
              onResponse={handleSeedInitResponse}
            />
          )}
          {showIntroduction && <IntroductionModal />}
          <Router>
            <Navigation />
            <InnerContent />
            <UpdaterDialog />
          </Router>
        </GlobalSnackbarProvider>
      </ThemeProvider>
    </StyledEngineProvider>
  );
}

function InnerContent() {
  return (
    <Box
      sx={{
        padding: 4,
        marginLeft: drawerWidth,
        maxHeight: `100vh`,
        flex: 1,
      }}
    >
      <Routes>
        <Route path="/swap" element={<SwapPage />} />
        <Route path="/history" element={<HistoryPage />} />
        <Route path="/wallet" element={<WalletPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/feedback" element={<FeedbackPage />} />
        <Route path="/" element={<SwapPage />} />
      </Routes>
    </Box>
  );
}

import { createTheme, adaptV4Theme } from "@mui/material";
import { indigo } from "@mui/material/colors";

export enum Theme {
  Light = "light",
  Dark = "dark",
  Darker = "darker"
}

const darkTheme = createTheme(adaptV4Theme({
  palette: {
    mode: "dark",
    primary: {
      main: "#f4511e", // Monero orange
    },
    secondary: indigo,
  },
  typography: {
    overline: {
      textTransform: "none", // This prevents the text from being all caps
      fontFamily: "monospace"
    },
  },
}));

const lightTheme = createTheme(adaptV4Theme({
  ...darkTheme,
  palette: {
    mode: "light",
    primary: {
      main: "#f4511e", // Monero orange
    },
    secondary: indigo,
  },
}));

const darkerTheme = createTheme(adaptV4Theme({
  ...darkTheme,
  palette: {
    mode: 'dark',
    primary: {
      main: "#f4511e",
    },
    secondary: indigo,
    background: {
      default: "#080808",
      paper: "#181818",
    },
  },
}));

export const themes = {
  [Theme.Dark]: darkTheme,
  [Theme.Light]: lightTheme,
  [Theme.Darker]: darkerTheme,
};

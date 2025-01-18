import { createTheme } from "@mui/material/styles";

const theme = createTheme({
  palette: {
    primary: {
      main: "#3f51b5", // Indigo color for primary
    },
    secondary: {
      main: "#f50057", // Pink color for secondary
    },
  },
  typography: {
    fontFamily: "Roboto, Arial, sans-serif",
    h1: {
      fontSize: "2.5rem",
      fontWeight: 300,
    },
    h2: {
      fontSize: "2rem",
      fontWeight: 400,
    },
    body1: {
      fontSize: "1rem",
    },
  },
  components: {
    MuiContainer: {
      styleOverrides: {
        root: {
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "16px",
        },
      },
    },
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: "none",
          margin: "8px",
          padding: "10px 20px",
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          padding: "16px",
          margin: "16px 0",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
        },
      },
    },
    MuiFormControl: {
      styleOverrides: {
        root: {
          margin: "8px",
          minWidth: "120px",
        },
      },
    },
    MuiInputLabel: {
      styleOverrides: {
        root: {
          color: "#3f51b5",
        },
      },
    },
    MuiMenuItem: {
      styleOverrides: {
        root: {
          padding: "10px 20px",
        },
      },
    },
    MuiSelect: {
      styleOverrides: {
        root: {
          minWidth: "120px",
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          margin: "8px",
          width: "100%",
        },
      },
    },
    MuiInputBase: {
      styleOverrides: {
        input: {
          padding: "10px 12px",
        },
      },
    },
    MuiOutlinedInput: {
      styleOverrides: {
        root: {
          "& $notchedOutline": {
            borderColor: "#3f51b5",
          },
          "&$focused $notchedOutline": {
            borderColor: "#f50057",
          },
        },
        notchedOutline: {},
      },
    },
    MuiFormLabel: {
      styleOverrides: {
        root: {
          "&$focused": {
            color: "#f50057",
          },
        },
        focused: {},
      },
    },
    MuiTypography: {
      styleOverrides: {
        root: {
          margin: "8px",
        },
      },
    },
  },
});

export default theme;

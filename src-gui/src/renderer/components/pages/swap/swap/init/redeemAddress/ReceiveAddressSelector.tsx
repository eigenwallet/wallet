import { Autocomplete, Box, TextField } from "@mui/material";
import { useState, useEffect } from "react";
import { getMoneroAddresses } from "renderer/rpc";

export default function ReceiveAddressSelector({
  onChange,
  value,
  isValidAddress,
}: {
  onChange: (address: string) => void;
  value: string;
  isValidAddress: boolean;
}) {
  const showError = value && !isValidAddress;
  const [addresses, setAddresses] = useState<string[]>([]);

  useEffect(() => {
    const fetchAddresses = async () => {
      const response = await getMoneroAddresses();
      setAddresses(response.addresses);
    };
    fetchAddresses();
  }, []);

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 2,
        width: "100%",
        marginTop: 1,
      }}
    >
      <Autocomplete
        sx={{
          flexGrow: 1,
        }}
        freeSolo
        options={addresses}
        value={value}
        onChange={(_, value) => onChange(value)}
        onInputChange={(_, value) => onChange(value)}
        renderInput={(params) => (
          <TextField
            {...params}
            label="Receive Address"
            fullWidth
            error={showError}
            helperText={showError ? "Invalid Monero address" : ""}
          />
        )}
      />
    </Box>
  );
}

import { Box, Button } from "@mui/material";
import { useAppDispatch } from "store/hooks";
import { swapReset } from "store/features/swapSlice";

export default function DefaultButtonSet() {
  const dispatch = useAppDispatch();

  function onCancel() {
    dispatch(swapReset());
  }

  return (
    <Box sx={{ display: "flex", justifyContent: "flex-end", width: "100%" }}>
      <Button variant="contained" onClick={onCancel}>
        Cancel
      </Button>
    </Box>
  );
}

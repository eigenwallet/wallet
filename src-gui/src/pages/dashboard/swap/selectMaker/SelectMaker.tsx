import {
  Typography,
  Box,
  DialogContent,
  DialogActions,
  Button,
} from "@mui/material";
import MakerOfferItem from "../components/MakerOfferItem";
import SwapOverview from "../components/SwapOverview";
import { useAppSelector, useAllMakers, useAppDispatch } from "store/hooks";
import { setSelectedMaker } from "store/features/makersSlice";
import { Maker } from "models/apiModel";
import {
  ListSellersDialogOpenButton,
  MakerSubmitDialogOpenButton,
} from "renderer/components/modal/provider/MakerListDialog";

export default function SelectMaker({
  onNext,
  onBack,
}: {
  onNext: () => void;
  onBack: () => void;
}) {
  const makers = useAllMakers();
  const selectedMaker = useAppSelector((state) => state.makers.selectedMaker);
  const dispatch = useAppDispatch();

  if (
    makers === null ||
    makers === undefined ||
    selectedMaker === null ||
    selectedMaker === undefined
  ) {
    return <div>Loading...</div>;
  }

  function handleSelectMaker(maker: Maker) {
    dispatch(setSelectedMaker(maker));
  }

  return (
    <>
      <DialogContent>
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            gap: 2,
          }}
        >
          <SwapOverview />
          <Box
            sx={{
              display: "flex",
              flexDirection: "row",
              gap: 1,
            }}
          >
            <Typography variant="h3">Select a Maker</Typography>
            <ListSellersDialogOpenButton />
          </Box>
          <Typography variant="body1">Best offer</Typography>
          <MakerOfferItem
            maker={selectedMaker!}
            checked={true}
            onSelect={handleSelectMaker}
          />
          <Typography variant="body1">Other offers</Typography>
          <Box
            sx={{
              display: "flex",
              flexDirection: "column",
              gap: 1,
            }}
          >
            {makers.map((maker) => (
              <MakerOfferItem
                key={maker.peerId}
                maker={maker}
                checked={selectedMaker?.peerId === maker.peerId}
                onSelect={handleSelectMaker}
              />
            ))}
          </Box>
          <MakerSubmitDialogOpenButton />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onBack}>
          Back
        </Button>
        <Button variant="contained" color="primary" onClick={onNext}>
          Get Offer
        </Button>
      </DialogActions>
    </>
  );
}

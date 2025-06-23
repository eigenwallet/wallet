import { Paper, Typography, Box } from "@mui/material";
import RedeemAddress from "./redeemAddress/RedeemAddress";
import WalletAndMakers from "./walletAndMakers/WalletAndMakers";
import Offer from "./offer/Offer";
import { useAppDispatch, useAppSelector } from "store/hooks";
import { setStep, StartSwapStep } from "store/features/startSwapSlice";
import { isXmrAddressValid } from "utils/conversionUtils";
import { isTestnet } from "store/config";

export default function SwapWidget() {
  const step = useAppSelector((state) => state.startSwap.step);
  const redeemAddress = useAppSelector(
    (state) => state.startSwap.redeemAddress,
  );

  const isValidAddress =
  redeemAddress && isXmrAddressValid(redeemAddress, isTestnet());

  const dispatch = useAppDispatch();
  const handleNext = () => {
    if (step === StartSwapStep.RedeemAddress) {
      dispatch(setStep(StartSwapStep.WalletAndMakers));
    } else if (step === StartSwapStep.WalletAndMakers) {
      dispatch(setStep(StartSwapStep.ReviewOffer));
    }
  };

  const handleBack = () => {
    if (step === StartSwapStep.WalletAndMakers) {
      dispatch(setStep(StartSwapStep.RedeemAddress));
    } else if (step === StartSwapStep.ReviewOffer) {
      dispatch(setStep(StartSwapStep.WalletAndMakers));
    }
  };

  return (
    <Paper
      elevation={3}
      sx={{
        width: "100%",
        maxWidth: 800,
        margin: "0 auto",
        borderRadius: 2,
      }}
    >
      <Box sx={{ padding: 3 }}>
        {!isValidAddress && <RedeemAddress onNext={handleNext} />}
        {isValidAddress && step === StartSwapStep.WalletAndMakers && (
          <WalletAndMakers onNext={handleNext} onBack={handleBack} />
        )}
        {isValidAddress && step === StartSwapStep.ReviewOffer && (
          <Offer onNext={handleNext} onBack={handleBack} />
        )}
      </Box>
    </Paper>
  );
}

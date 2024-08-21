import { ReactNode } from "react";
import { Box, Typography } from "@material-ui/core";
import FileCopyOutlinedIcon from "@material-ui/icons/FileCopyOutlined";
import InfoBox from "./InfoBox";
import ClipboardIconButton from "./ClipbiardIconButton";
import BitcoinQrCode from "./BitcoinQrCode";
import ClickableAddress from "renderer/components/other/ClickableAddress";

type Props = {
  title: string;
  address: string;
  additionalContent: ReactNode;
  icon: ReactNode;
};

export default function DepositAddressInfoBox({
  title,
  address,
  additionalContent,
  icon,
}: Props) {
  return (
    <InfoBox
      title={title}
      mainContent={<ClickableAddress address={address} />}
      additionalContent={
        <Box
          style={{
            display: "flex",
            flexDirection: "row",
            gap: "0.5rem",
            alignItems: "center",
          }}
        >
          <Box>{additionalContent}</Box>
          <BitcoinQrCode address={address} />
        </Box>
      }
      icon={icon}
      loading={false}
    />
  );
}

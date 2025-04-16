import { Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from '../../../../../assets/configureSwap.png' 

export default function Slide02_ChooseAMaker(props: slideProps) {
    return (
        <SlideTemplate title="Prepare Swap" stepLabel="Step 2" {...props} imagePath={imagePath} imagePadded>
            <Typography variant="subtitle1">
                After providing your Monero redeem address and configuring the
                Bitcoin refund behavior, you can initiate the swap and deposit
                funds in the internal Bitcoin wallet.
            </Typography>
        </SlideTemplate>
    )
}

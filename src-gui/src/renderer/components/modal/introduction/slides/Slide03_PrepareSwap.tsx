import { Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from 'assets/mockConfigureSwap.svg' 

export default function Slide02_ChooseAMaker(props: slideProps) {
    return (
        <SlideTemplate title="Prepare Swap" stepLabel="Step 2" {...props} imagePath={imagePath}>
            <Typography variant="subtitle1">
            After providing your redeem address and (optionally) providing a refund address,
            you can initiate the swap by deposit your Bitcoin.
            </Typography>
        </SlideTemplate>
    )
}

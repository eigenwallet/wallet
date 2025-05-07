import { Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from 'assets/simpleSwapFlowDiagram.svg'

export default function Slide02_ChooseAMaker(props: slideProps) {
    return (
        <SlideTemplate
            title="Execute Swap"
            stepLabel="Step 3"
            {...props}
            imagePath={imagePath}
        >
            <Typography variant="subtitle1">
                By confirming, the swap begins:
            </Typography>
            <Typography>
                <ol>
                    <li>Your Bitcoin funds are locked</li>
                    <li>The maker locks monero</li>
                    <li>The maker reedems the BTC</li>
                    <li>The Monero is redeemed to your payout address</li>
                </ol>
            </Typography>
        </SlideTemplate>
    )
}

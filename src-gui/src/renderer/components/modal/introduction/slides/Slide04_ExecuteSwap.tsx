import { Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from '../../../../../assets/simpleSwapFlowDiagram.webp'

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
                    <li>Your BTC funds are locked</li>
                    <li>The maker locks monero</li>
                    <li>The maker reedems the BTC</li>
                    <li>
                        The swaped Monero is redeemed and sent to your address
                    </li>
                </ol>
            </Typography>
        </SlideTemplate>
    )
}

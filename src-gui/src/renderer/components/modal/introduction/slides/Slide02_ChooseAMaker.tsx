import { Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from '../../../../../assets/mockMakerSelection.webp'

export default function Slide02_ChooseAMaker(props: slideProps) {
    return (
        <SlideTemplate
            title="Choose a Maker"
            stepLabel="Step 1"
            {...props}
            imagePath={imagePath}
        >
            <Typography variant="subtitle1">
                Makers provide liquidity in XMR which can be exchanged for
                BTC. They have varying Min and Max amounts they can swap and
                charge varying markup fees. Makers can be found with
            </Typography>
            <Typography>
                <ul>
                    <li>the <strong>Public Registry</strong></li>
                    <li>by connecting to a <strong>Rendezvous Points</strong></li>
                </ul>
            </Typography>
        </SlideTemplate>
    )
}

import { Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from 'assets/mockMakerSelection.webp'

export default function Slide02_ChooseAMaker(props: slideProps) {
    return (
        <SlideTemplate
            title="Choose a Maker"
            stepLabel="Step 1"
            {...props}
            imagePath={imagePath}
        >
            <Typography variant="subtitle1">
                To start a Swap, choose a maker to exchange Bitcoin for Monero.
                The app automatically shows available makers from the public registry, including their swap limits and uptime.
            </Typography>

            <Typography
                variant="caption"
                color="textSecondary"
                style={{ marginTop: 8 }}
            >
                Additionally, you can find makers through rendezvous points or direct connections.
            </Typography>
        </SlideTemplate>
    )
}

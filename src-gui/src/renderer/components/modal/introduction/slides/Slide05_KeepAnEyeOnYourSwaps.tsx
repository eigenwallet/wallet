import { Link, Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from 'assets/mockHistoryPage.webp'
import ExternalLink from 'renderer/components/other/ExternalLink'

export default function Slide05_KeepAnEyeOnYourSwaps(props: slideProps) {
    return (
        <SlideTemplate
            title="Monitor Your Swaps"
            stepLabel="Step 3"
            {...props}
            imagePath={imagePath}
        >
            <Typography>
                While the Atomic Swap Protocol is secure, you should monitor active swaps in the history tab to ensure everything proceeds smoothly.
            </Typography>
            <Typography>
                <ExternalLink href='https://docs.unstoppableswap.net/usage/first_swap'>
                    Learn more about atomic swaps
                </ExternalLink>
            </Typography>
        </SlideTemplate>
    )
}

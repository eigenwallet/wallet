import { Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from '../../../../../assets/walletWithBitcoinAndMonero.png'

export default function Slide01_GettingStarted(props: slideProps) {
    return (
        <SlideTemplate
            title="Getting Started"
            {...props}
            imagePath={imagePath}
            customContinueButtonText="Start"
        >
            <Typography variant="subtitle1">
                To make a Atomic Swap with Unstoppable swap you need a
            </Typography>
            <Typography>
                <ul>
                    <li><strong>Bitcoin Wallet</strong> with <strong>Funds</strong> for the swap</li>
                    <li>Monero Wallet to generate a <strong>Monero Redeem Address</strong></li>
                </ul>
            </Typography>
        </SlideTemplate>
    )
}

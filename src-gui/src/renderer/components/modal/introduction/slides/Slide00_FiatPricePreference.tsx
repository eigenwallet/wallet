import { Box, Typography, Paper, Button } from '@material-ui/core'
import CardSelectionGroup from 'renderer/components/inputs/CardSelection/CardSelectionGroup'
import CardSelectionOption from 'renderer/components/inputs/CardSelection/CardSelectionOption'
import logo from 'assets/unstoppableSwapLogo.png'

const FiatPricePreferenceSlide = ({
    handleContinue,
    showFiat,
    onChange,
}: slideProps & {
    showFiat: boolean
    onChange: (value: string) => void
}) => {
    return (
        <Paper
            style={{
                height: '80%',
                width: '80%',
                display: 'flex',
                justifyContent: 'space-between',
            }}
        >
            {/* Left Side */}
            <Box
                m={3}
                flex
                alignContent="center"
                position="relative"
                width="50%"
                flexGrow={1}
                display="flex"
                flexDirection="column"
                justifyContent="center"
            >
                <Typography variant="h4" gutterBottom>
                    Welcome to Unstoppable Swap
                </Typography>
                <Typography variant="subtitle1" color="textSecondary">
                    Do you want to show fiat prices?
                </Typography>
                <CardSelectionGroup
                    value={showFiat ? 'show' : 'hide'}
                    onChange={onChange}
                >
                    <CardSelectionOption value="show">
                        <Typography>Show fiat prices</Typography>
                        <Typography
                            variant="caption"
                            color="textSecondary"
                            paragraph
                        >
                            We connect to CoinGecko to provide realtime currency
                            prices.
                        </Typography>
                        <Typography variant="caption" color="textSecondary">
                            Tip: Use a VPN to remain completely anonymous.
                        </Typography>
                    </CardSelectionOption>
                    <CardSelectionOption value="hide">
                        <Typography>Don't show fiat prices</Typography>
                    </CardSelectionOption>
                </CardSelectionGroup>
                <Typography
                    variant="caption"
                    color="textSecondary"
                    style={{ marginTop: 8 }}
                >
                    You can change your preference later in the settings
                </Typography>
                <Box
                    position="absolute"
                    bottom={0}
                    width="100%"
                    display="flex"
                    justifyContent="flex-end"
                >
                    <Button
                        onClick={handleContinue}
                        variant="contained"
                        color="primary"
                    >
                        Next
                    </Button>
                </Box>
            </Box>
            {/* Right Side */}
            <Box
                width="50%"
                display="flex"
                flexDirection="column"
                alignItems="center"
                justifyContent="center"
                bgcolor="#232323"
            >
                <img src={logo} alt="UnstoppableSwap" style={{ borderRadius: 16, width: 160, height: 160 }} />
                <Typography
                    variant="h5"
                    style={{ color: '#fff', fontWeight: 700, marginTop: 24 }}
                    gutterBottom
                >
                    UnstoppableSwap
                </Typography>
                <Typography
                    variant="subtitle1"
                    style={{ color: '#fff', opacity: 0.7, lineHeight: 1.5 }}
                    align="center"
                >
                    Exchange Bitcoin for Monero.
                    <br />
                    Secure and Free.
                </Typography>
            </Box>
        </Paper>
    )
}

export default FiatPricePreferenceSlide

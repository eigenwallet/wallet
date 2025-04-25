import {
    Box,
    Button,
    Link,
    makeStyles,
    Paper,
    TableCell,
    TableHead,
    TableRow,
    Typography,
} from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import SwapStatusAlert from 'renderer/components/alert/SwapStatusAlert/SwapStatusAlert'
import { BobStateName, GetSwapInfoResponseExt } from 'models/tauriModelExt'
import { open } from "@tauri-apps/plugin-shell";

const useStyles = makeStyles({
    paper: {
        height: '80%',
        width: '80%',
        display: 'flex',
        justifyContent: 'space-between',
    },
    textWrapper: {
        display: 'flex',
        flexDirection: 'column',
        gap: '1em',
    },
    headingSwapAlert: {
        padding: "1em",
        marginBottom: "0.5em",
        backgroundColor: "#313131",
        width: "100%",
        borderRadius: "0.4em"
    }
})

const dummySwap: GetSwapInfoResponseExt = {
    swap_id: 'TEST_SWAP0000111111',
    seller: {
        peer_id: 'string',
        addresses: ['string'],
    },
    completed: false,
    start_date: 'string',
    xmr_amount: 0,
    btc_amount: 0,
    tx_lock_id: 'string',
    tx_cancel_fee: 0,
    tx_refund_fee: 0,
    tx_lock_fee: 0,
    btc_refund_address: 'string',
    cancel_timelock: 0,
    punish_timelock: 0,
    state_name: BobStateName.BtcLocked,
    timelock: {
        type: 'None',
        content: {
            blocks_left: 0,
        },
    },
}

export default function Slide05_KeepAnEyeOnYourSwaps({
    handleContinue,
    handlePrevious,
    hidePreviousButton,
}: slideProps) {
    const classes = useStyles()

    return (
        <Paper className={classes.paper}>
            <Box
                m={3}
                flex
                alignContent="center"
                position="relative"
                width="50%"
                flexGrow={1}
            >
                <Box className={classes.textWrapper}>
                    <Typography variant="h5">
                        Have an eye on your unfinished swaps
                    </Typography>
                    <Typography>
                        The Atomic Swap Protocol is safe and reliable, but if
                        something doesn't work as expected during the process
                        you need to get active to avoid losing your funds.
                    </Typography>
                    <Typography>
                        Therefore it's important that you regularly check your
                        active swaps in the history tab.{' '}
                        <Link onClick={() => open('https://docs.unstoppableswap.net/usage/first_swap')}>Further Information</Link>
                    </Typography>
                    <Typography>
                        Using our Swap History, that's super easy:
                    </Typography>
                </Box>
                <Box
                    position="absolute"
                    bottom={0}
                    width="100%"
                    display="flex"
                    justifyContent={
                        hidePreviousButton ? 'flex-end' : 'space-between'
                    }
                >
                    {!hidePreviousButton && (
                        <Button onClick={handlePrevious}>Previous</Button>
                    )}
                    <Button
                        onClick={handleContinue}
                        variant="contained"
                        color="primary"
                    >
                        Continue
                    </Button>
                </Box>
            </Box>
            <Box
                p="1em"
                bgcolor="#212121"
                width="50%"
                height="100%"
                display="flex"
                justifyContent="center"
                alignItems="center"
                flexDirection="column"
            >
                <Paper elevation={2} className={classes.headingSwapAlert}>
                        <Typography>Swap History</Typography>
                </Paper>
                <Box height="auto">
                    <SwapStatusAlert
                        swap={dummySwap}
                        isRunning
                    ></SwapStatusAlert>
                </Box>
            </Box>
        </Paper>
    )
}

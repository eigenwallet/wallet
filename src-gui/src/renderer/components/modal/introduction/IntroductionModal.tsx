import {
    Box,
    Button,
    Container,
    Grid,
    List,
    ListItem,
    ListItemText,
    makeStyles,
    Modal,
    Paper,
    Typography,
} from '@material-ui/core'


const useStyles = makeStyles({
    modal: {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
    },
    paper: {
        width: '80%',
        display: 'flex',
        justifyContent: 'space-between'
    }
})

export default function IntroductionModal() {
    const classes = useStyles()

    return (
        <Modal open={true} className={classes.modal}>
            <Paper className={classes.paper}>
                <Box m={3} flex alignContent="center" position="relative">
                    <Box>
                        <Typography variant="h3">Getting Started</Typography>
                        <Typography variant="subtitle1">
                            To make a Atomic Swap with Unstoppable swap you need
                            a
                        </Typography>
                        <ul>
                            <li>Bitcoin Wallet with Funds for the swap</li>
                            <li>
                                Monero Wallet to generate a Monero Redeem
                                Address
                            </li>
                        </ul>
                    </Box>
                    <Box position="absolute" bottom={0} width="100%" display="flex" justifyContent="space-between">
                        <Button>Previous</Button>
                        <Button variant="contained" color="primary">Continue</Button>
                    </Box>
                </Box>
                <Box bgcolor='#212121' width='50%' height="600px"></Box>
            </Paper>
        </Modal>
    )
}

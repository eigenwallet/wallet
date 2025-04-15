import {
    Box,
    Button,
    makeStyles,
    Modal,
    Paper,
    Typography,
} from '@material-ui/core'
import { useState } from 'react'

const useStyles = makeStyles({
    modal: {
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
    },
    paper: {
        width: '80%',
        display: 'flex',
        justifyContent: 'space-between',
    },
})

type slideProps = {
    handleContinue: () => void
    handlePrevious: () => void
    hidePreviousButton?: boolean
}

function FirstSlide({
    handleContinue,
    handlePrevious,
    hidePreviousButton,
}: slideProps) {
    const classes = useStyles()

    return (
        <>
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
                <Box bgcolor="#212121" width="50%" height="600px"></Box>
            </Paper>
        </>
    )
}

function SecondSlide({ handleContinue, handlePrevious }: slideProps) {
    const classes = useStyles()

    return (
        <>
            <Paper className={classes.paper}>
                <Box m={3} flex alignContent="center" position="relative">
                    <Box>
                        <Typography variant="h3">
                            This is the second Slide!
                        </Typography>
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
                    <Box
                        position="absolute"
                        bottom={0}
                        width="100%"
                        display="flex"
                        justifyContent="space-between"
                    >
                        <Button onClick={handlePrevious}>Previous</Button>
                        <Button
                            onClick={handleContinue}
                            variant="contained"
                            color="primary"
                        >
                            Continue
                        </Button>
                    </Box>
                </Box>
                <Box bgcolor="#212121" width="50%" height="600px"></Box>
            </Paper>
        </>
    )
}

export default function IntroductionModal() {
    // Handle Display State
    const [open, setOpen] = useState<boolean>(true)

    const handleClose = () => {
        setOpen(false)
    }

    // Handle Slide Index
    const [currentSlideIndex, setCurrentSlideIndex] = useState(0)

    const handleContinue = () => {
        if (currentSlideIndex == slideComponents.length - 1) {
            handleClose()
            return
        }

        setCurrentSlideIndex((i) => i + 1)
    }

    const handlePrevious = () => {
        if (currentSlideIndex == 0) {
            return
        }

        setCurrentSlideIndex((i) => i - 1)
    }

    const slideComponents = [
        <FirstSlide
            handleContinue={handleContinue}
            handlePrevious={handlePrevious}
            hidePreviousButton
        />,
        <SecondSlide
            handleContinue={handleContinue}
            handlePrevious={handlePrevious}
        />,
    ]

    const classes = useStyles()

    return (
        <Modal
            open={open}
            onClose={handleClose}
            className={classes.modal}
            disableAutoFocus
            closeAfterTransition
        >
            {slideComponents[currentSlideIndex]}
        </Modal>
    )
}

import {
    makeStyles,
    Modal,
} from '@material-ui/core'
import { useState } from 'react'
import Slide01_GettingStarted from './slides/Slide01_GettingStarted'
import Slide02_ChooseAMaker from './slides/Slide02_ChooseAMaker'
import Slide03_PrepareSwap from './slides/Slide03_PrepareSwap'
import Slide04_ExecuteSwap from './slides/Slide04_ExecuteSwap'
import Slide06_ReachOut from './slides/Slide06_ReachOut'
import Slide05_KeepAnEyeOnYourSwaps from './slides/Slide05_KeepAnEyeOnYourSwaps'

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
        <Slide01_GettingStarted
            handleContinue={handleContinue}
            handlePrevious={handlePrevious}
            hidePreviousButton
        />,
        <Slide02_ChooseAMaker
            handleContinue={handleContinue}
            handlePrevious={handlePrevious}
        />,
        <Slide03_PrepareSwap
            handleContinue={handleContinue}
            handlePrevious={handlePrevious}
        />,
        <Slide04_ExecuteSwap
            handleContinue={handleContinue}
            handlePrevious={handlePrevious}
        />,
        <Slide05_KeepAnEyeOnYourSwaps
            handleContinue={handleContinue}
            handlePrevious={handlePrevious}
        />,
        <Slide06_ReachOut
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

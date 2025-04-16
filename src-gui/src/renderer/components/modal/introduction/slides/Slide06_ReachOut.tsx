import { Box, Icon, IconButton, Typography } from '@material-ui/core'
import SlideTemplate from './SlideTemplate'
import imagePath from '../../../../../assets/groupWithChatbubbles.png' 
import GitHubIcon from "@material-ui/icons/GitHub"
import MatrixIcon from 'renderer/components/icons/MatrixIcon'

export default function Slide02_ChooseAMaker(props: slideProps) {
    return (
        <SlideTemplate title="Reach out" {...props} imagePath={imagePath} customContinueButtonText="Start using the app">
            <Typography variant="subtitle1">
                We would love to hear about your experience with Unstoppable
                Swap and invite you to join our community.
            </Typography>
            <Box mt={3}>
                <IconButton >
                    <GitHubIcon/>
                </IconButton>
                <IconButton>
                    <MatrixIcon/>
                </IconButton>
            </Box>
        </SlideTemplate>
    )
}

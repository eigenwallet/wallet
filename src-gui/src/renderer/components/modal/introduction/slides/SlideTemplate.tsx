import { makeStyles, Paper, Box, Typography, Button } from "@material-ui/core"

type slideProps = {
    handleContinue: () => void
    handlePrevious: () => void
    hidePreviousButton?: boolean
    stepLabel?: String
    title: String
    children?: React.ReactNode
    rightSideImage?: React.ReactNode
}

const useStyles = makeStyles({
    paper: {
        width: '80%',
        display: 'flex',
        justifyContent: 'space-between',
    },
    stepLabel: {
        textTransform: 'uppercase'
    }
})

export default function SlideTemplate({
    handleContinue,
    handlePrevious,
    hidePreviousButton,
    stepLabel,
    title,
    children,
    rightSideImage,
}: slideProps) {
    const classes = useStyles()

    return (
        <Paper className={classes.paper}>
            <Box m={3} flex alignContent="center" position="relative">
                <Box>
                    {stepLabel && <Typography variant="overline" className={classes.stepLabel}>{stepLabel}</Typography>}
                    <Typography variant="h3">{title}</Typography>
                    {children}
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
            <Box bgcolor="#212121" width="50%" height="600px">
                {rightSideImage}
            </Box>
        </Paper>
    )
}

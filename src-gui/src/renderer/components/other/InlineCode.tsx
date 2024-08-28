import { Box, makeStyles } from "@material-ui/core"
import { ReactNode } from "react"

type Props = {
    content: string,
    onClick?: (content: string) => void,
    endIcon?: ReactNode
}

const useStyles = makeStyles(theme => ({
    root: {
        display: "inline-flex",
        alignItems: 'center',
        backgroundColor: theme.palette.grey[900],
        borderRadius: theme.shape.borderRadius,
        padding: theme.spacing(0.5)
    }
}))

export default function InlineCode({ content, endIcon, onClick }: Props) {
    // Use custom style 
    const classes = useStyles()

    // Call onClick if specified
    const handleClick = () => {
        if (onClick !== undefined) {
            onClick(content)
        }
    }

    return (
        <Box className={classes.root} onClick={handleClick}>
            {content}
            {endIcon}
        </Box>
    )
}

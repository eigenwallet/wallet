import { Box } from "@material-ui/core";

export default function CardSelectionOption({children, value}: {children: React.ReactNode, value: string}) {
    return (
        <Box>
            {children}
        </Box>
    )
}
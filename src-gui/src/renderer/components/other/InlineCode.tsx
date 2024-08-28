import { ReactNode } from 'react';
import { Button, Typography } from '@material-ui/core';

type Props = {
    onClick: (e: any) => void;
    content: string,
    icon?: ReactNode | null
}

export default function InlineCode({onClick, content, icon}: Props) {
    return (
        <Button 
            variant='outlined' 
            endIcon={icon}
            onClick={onClick}
        >
            <Typography variant="overline">
                {content}
            </Typography>
        </Button>
    )
}

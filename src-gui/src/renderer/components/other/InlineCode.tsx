import { ReactNode } from 'react';
import styles from './InlineCode.module.css'

type Props = {
    onClick: (e: any) => void;
    content: string,
    icon?: ReactNode | null
}

export default function InlineCode({onClick, content, icon}: Props) {
    return (
        <span 
            className={styles.container} 
            onClick={onClick}
        >
            {content}
            {" "}
            {icon}
        </span>
    )
}

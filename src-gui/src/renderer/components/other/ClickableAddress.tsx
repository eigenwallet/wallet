import React, { useState } from "react";
import { Tooltip } from "@material-ui/core";
import InlineCode from "./InlineCode";
import { FileCopy, FileCopyOutlined } from "@material-ui/icons";

type Props = {
    address: string
}

/** Display addresses monospaced and clickable such that a click copies the address to the clipboard. */
export default function ClickableAddress({address}: Props) {
    // Signal that the address was copied
    const [copied, setCopied] = useState(false);
    const tooltip = copied ? "copied" : "copy";

    // Copy address to clipboard on-click
    const handleClick = async () => {
        // copy to clipboard
        await navigator.clipboard.writeText(address);
        // change tooltip to show that we copied the address
        setCopied(true)
        // after a delay, show default tooltip again (2sec)
        setTimeout(() => setCopied(false), 2_000)
    }

    return (
        <Tooltip 
            title={tooltip}
            arrow
        >
            {/* div is necessary to make the tooltip work */}
            <div>
                <InlineCode 
                    content={address} 
                    onClick={handleClick} 
                    icon={<FileCopyOutlined />}
                />
            </div>
        </Tooltip>

    )
}


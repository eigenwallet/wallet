import fs from "fs";
import path from "path";

export function checkObfs4proxyPath(path: string): boolean {
    if (!path)
        return false;

    // Check if the path exists and is a file
    const stat = fs.statSync(path);
    if (!stat.isFile())
        return false;

    // Check that the file name includes "obfs4proxy"
    if (!path.includes("obfs4proxy"))
        return false;

    return true;
}

export function proposeObfs4proxyPath(): string[] {
    const paths = []

    // Todo: add default paths depending on the OS

    return paths;
}

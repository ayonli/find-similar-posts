import { strictEqual } from "node:assert";
// @ts-types="../index.d.ts"
import { PostData } from "../index.js";
import { findSimilarPosts } from "./mod.ts";

Deno.test("findClosetPostsTopN", () => {
    const source: PostData = {
        title: "Deno.kill not working on windows",
        content: `
Version: Deno 2.3.3
OS: Windows 11

Sending a SIGINT OS signal on windows like: Deno.kill(Deno.pid, 'SIGINT');
Results in: TypeError: Windows only supports ctrl-c (SIGINT) and ctrl-break (SIGBREAK), but got SIGINT
        `,
    };

    const candidates: PostData[] = [
        {
            title: "Deno.kill on windows",
            content: `
Version: Deno 2.3.3
OS: Windows 11

Sending a SIGINT OS signal on windows like: Deno.kill(Deno.pid, 'SIGINT');
Results in: TypeError: Windows only supports ctrl-c (SIGINT) and ctrl-break (SIGBREAK), but got SIGINT

Same goes for SIGBREAK

Registering event listeners with: Deno.addSignalListener('SIGINT', doSomething); Works correctly
            `,
        },
        {
            title: "denojs on termux like nodejs",
            content: `
We want a smooth download for Deno.js like Node.js, Python, etc., instead of downloading extra
libraries on Termux. We seek a streamlined download process for Deno.js similar to that of Node.js
and Python, rather than having to download additional libraries on Termux.
            `,
        },
    ];

    const { matches } = findSimilarPosts(source, candidates, 1);
    strictEqual(matches.length, 1);
    strictEqual(matches[0].target.title, "Deno.kill on windows");
});

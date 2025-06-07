import { chunk } from "@ayonli/jsext/array";
import { avg } from "@ayonli/jsext/math";
import parallel from "@ayonli/jsext/parallel";
// @deno-types="../index.d.ts"
import { FindTopNResult, PostData } from "../index.js";
const { findSimilarPosts } = parallel(() => import("./mod.ts"));

export async function findSimilarPostsParallel(
    source: PostData,
    candidates: PostData[],
    topN: number,
): Promise<FindTopNResult> {
    const chunks = chunk(
        candidates,
        Math.ceil(candidates.length / navigator.hardwareConcurrency),
    );
    const processTimes: number[] = [];
    let results = (await Promise.all(
        chunks.map((chunk) => findSimilarPosts(source, chunk, topN)),
    ))
        .flatMap((e) => {
            processTimes.push(e.processTime);
            return e.matches;
        })
        .toSorted((a, b) => b.score - a.score);

    if (results.length > topN) {
        results = results.slice(0, topN);
    }

    return {
        matches: results,
        processTime: Math.round(avg(...processTimes)),
    };
}

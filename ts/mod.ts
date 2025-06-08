import { levenshteinDistance } from "@std/text"
import { chars } from "@ayonli/jsext/string"
// @deno-types="../index.d.ts"
import { FindTopNResult, Match, PostData } from "../index.js"

// @std/text doesn't provide a normalized function, we implement our own
function normalizedSimilarity(a: string, b: string): number {
    if (a === b) {
        return 1
    }

    const aChars = chars(a).length
    const bChars = chars(b).length
    return 1 - levenshteinDistance(a, b) / Math.max(aChars, bChars)
}

function getWeights(source: PostData): [number, number] {
    const titleChars = chars(source.title).length
    const contentChars = chars(source.content).length
    const totalChars = titleChars + contentChars

    if (totalChars === 0) {
        throw new Error("Source is invalid")
    }

    return [
        titleChars / totalChars,
        contentChars / totalChars,
    ]
}

export function findSimilarPosts(
    source: PostData,
    candidates: PostData[],
    topN: number,
): FindTopNResult {
    const start = Date.now()
    const [titleWeight, contentWeight] = getWeights(source)
    const matches: Match[] = []

    for (const candidate of candidates) {
        const titleScore = normalizedSimilarity(source.title, candidate.title) *
            titleWeight
        const contentScore =
            normalizedSimilarity(source.content, candidate.content) *
            contentWeight
        const score = titleScore + contentScore

        // 0.5 is the threshold to consider a match
        if (score > 0.5) {
            matches.push({
                target: candidate,
                score,
            })
        }
    }

    matches.sort((a, b) => b.score - a.score)

    return {
        matches: matches.length <= topN ? matches : matches.slice(0, topN),
        processTime: Date.now() - start,
    }
}

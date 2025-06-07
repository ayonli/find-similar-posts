import { deepStrictEqual, strictEqual } from "node:assert";
import { parse } from "@std/csv";
// @ts-types="./index.d.ts"
import {
    findSimilarPostsNative,
    findSimilarPostsNativeAsync,
    findSimilarPostsNativeParallel,
    Match,
    PostData,
} from "./index.js";
import { findSimilarPosts } from "./ts/mod.ts";
import { findSimilarPostsParallel } from "./ts/parallel.ts";

const filename = Deno.cwd() + "/assets/blogs.csv"; // the archive of the blog posts
const contents = await Deno.readTextFile(filename);
const records = parse(contents, { skipFirstRow: true }) as {
    title: string;
    text: string;
}[];

const posts: PostData[] = records.map((r) => ({
    title: r.title,
    content: r.text,
}));
console.log("Existing posts:", posts.length);

const newPost: PostData = {
    title: "The economic theories of MLA",
    content: `
A resolution calling for full-and-part-time faculty members to be eligible for tenure and expressing
the view that all higher education employees should have appropriate forms of job security, due
process, a living wage and access to health care benefits passed in a 81-15 vote, but not without
concerns from delegates that the wording went too far or not far enough.

Ian Barnard, an associate professor of English at California State University-Northridge, said he
wanted to see the resolution extended to include a call for all faculty to be eligible not only for
tenure but also for full-time employment. Simply voicing support for a lecturer to continue to be
guaranteed one course per semester was, he said, really weak a way for us to cop out, for
departments to avoid paying for health benefits and for adjunct faculty to continue bouncing around
among many jobs just to make ends meet.

The full story is here. Why don't journalists demand something similar? You can pinch yourself, but
it really is 2010. By the way, here are some facts:

In 1960, 75 percent of college instructors were full-time tenured or tenure-track professors; today
only 27 percent are.
    `,
};

interface FunctionResult {
    fn_name: string;
    call_time_ms: number;
    process_time_ms: number;
    data_passing_ms: number;
}

const results: FunctionResult[] = [];
let match: Match | undefined;

{
    const start = Date.now();
    const { matches, processTime } = findSimilarPosts(newPost, posts, 3);
    const callTime = Date.now() - start;
    results.push({
        fn_name: "findSimilarPosts",
        call_time_ms: callTime,
        process_time_ms: processTime,
        data_passing_ms: callTime - processTime,
    });
    strictEqual(matches.length, 1);
    strictEqual(matches[0].target.title, "The economic theories of the MLA");
    strictEqual(matches[0].score > 0.95, true);
    match = matches[0];
}

{
    const start = Date.now();
    const { matches, processTime } = findSimilarPostsNative(newPost, posts, 3);
    const callTime = Date.now() - start;
    results.push({
        fn_name: "findSimilarPostsNative",
        call_time_ms: callTime,
        process_time_ms: processTime,
        data_passing_ms: callTime - processTime,
    });
    strictEqual(matches.length, 1);
    deepStrictEqual(matches[0], match);
}

{
    const start = Date.now();
    const { matches, processTime } = await findSimilarPostsParallel(
        newPost,
        posts,
        3,
    );
    const callTime = Date.now() - start;
    results.push({
        fn_name: "findSimilarPostsParallel",
        call_time_ms: callTime,
        process_time_ms: processTime,
        data_passing_ms: callTime - processTime,
    });
    strictEqual(matches.length, 1);
    deepStrictEqual(matches[0], match);
}

{
    const start = Date.now();
    const { matches, processTime } = findSimilarPostsNativeParallel(
        newPost,
        posts,
        3,
    );
    const callTime = Date.now() - start;
    results.push({
        fn_name: "findSimilarPostsNativeParallel",
        call_time_ms: callTime,
        process_time_ms: processTime,
        data_passing_ms: callTime - processTime,
    });
    strictEqual(matches.length, 1);
    deepStrictEqual(matches[0], match);
}

{
    const start = Date.now();
    const { matches, processTime } = await findSimilarPostsNativeAsync(
        newPost,
        posts,
        3,
    );
    const callTime = Date.now() - start;
    results.push({
        fn_name: "findSimilarPostsNativeAsync",
        call_time_ms: callTime,
        process_time_ms: processTime,
        data_passing_ms: callTime - processTime,
    });
    strictEqual(matches.length, 1);
    deepStrictEqual(matches[0], match);
}

console.table(results);
Deno.exit(0);

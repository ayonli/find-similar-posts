#![deny(clippy::all)]
use std::time::Instant;

use napi::{bindgen_prelude::AsyncTask, Env, Error, Result, Task};
use rapidfuzz::distance::levenshtein::normalized_similarity;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[macro_use]
extern crate napi_derive;

#[napi(object)]
#[derive(Debug, Clone)]
pub struct PostData {
    pub title: String,
    pub content: String,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct Match {
    pub target: PostData,
    pub score: f64,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct FindTopNResult {
    pub matches: Vec<Match>,
    pub process_time: i64, // how many time is used for processing
}

fn get_weights(source: &PostData) -> Result<(f64, f64)> {
    let title_chars = source.title.chars().count();
    let content_chars = source.content.chars().count();
    let total_chars = title_chars + content_chars;

    if total_chars == 0 {
        Err(Error::from_reason("source is invalid"))
    } else {
        Ok((
            title_chars as f64 / total_chars as f64,
            content_chars as f64 / total_chars as f64,
        ))
    }
}

#[napi]
pub fn find_similar_posts_native(
    source: PostData,
    candidates: Vec<PostData>,
    top_n: u32,
) -> Result<FindTopNResult> {
    let start = Instant::now();
    let (title_weight, content_weight) = get_weights(&source)?;
    let mut matches = vec![];

    for candidate in candidates.iter() {
        let title_score =
            normalized_similarity(source.title.chars(), candidate.title.chars()) * title_weight;
        let content_score =
            normalized_similarity(source.content.chars(), candidate.content.chars())
                * content_weight;
        let score = title_score + content_score;

        // 0.5 is the threshold to consider a match
        if score > 0.5 {
            matches.push(Match {
                target: candidate.clone(),
                score,
            });
        }
    }

    matches.sort_by(|a, b| {
        let order = b.score.partial_cmp(&a.score);
        order.unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_n_matches = if matches.len() <= top_n as usize {
        matches
    } else {
        matches.into_iter().take(top_n as usize).collect()
    };
    let duration = start.elapsed();

    Ok(FindTopNResult {
        matches: top_n_matches,
        process_time: duration.as_millis() as i64,
    })
}

#[napi]
fn find_similar_posts_native_parallel(
    source: PostData,
    candidates: Vec<PostData>,
    top_n: u32,
) -> Result<FindTopNResult> {
    do_find_similar_posts_native_parallel(&source, &candidates, top_n)
}

fn do_find_similar_posts_native_parallel(
    source: &PostData,
    candidates: &Vec<PostData>,
    top_n: u32,
) -> Result<FindTopNResult> {
    let start = Instant::now();
    let (title_weight, content_weight) = get_weights(source)?;

    let matches: Vec<Match> = candidates
        .par_iter()
        .map(|candidate| {
            let title_score =
                normalized_similarity(source.title.chars(), candidate.title.chars()) * title_weight;
            let content_score =
                normalized_similarity(source.content.chars(), candidate.content.chars())
                    * content_weight;
            let score = title_score + content_score;

            if score > 0.5 {
                Some(Match {
                    target: candidate.clone(),
                    score,
                })
            } else {
                None
            }
        })
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .collect();
    let mut matches = matches;

    matches.sort_by(|a, b| {
        let order = b.score.partial_cmp(&a.score);
        order.unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_n_matches = if matches.len() <= top_n as usize {
        matches
    } else {
        matches.into_iter().take(top_n as usize).collect()
    };
    let duration = start.elapsed();

    Ok(FindTopNResult {
        matches: top_n_matches,
        process_time: duration.as_millis() as i64,
    })
}

pub struct AsyncFindSimilarPosts {
    source: PostData,
    candidates: Vec<PostData>,
    top_n: u32,
}

impl Task for AsyncFindSimilarPosts {
    type Output = FindTopNResult;
    type JsValue = FindTopNResult;

    fn compute(&mut self) -> Result<Self::Output> {
        do_find_similar_posts_native_parallel(&self.source, &self.candidates, self.top_n)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi(ts_return_type = "Promise<FindTopNResult>")]
pub fn find_similar_posts_native_async(
    source: PostData,
    candidates: Vec<PostData>,
    top_n: u32,
) -> AsyncTask<AsyncFindSimilarPosts> {
    AsyncTask::new(AsyncFindSimilarPosts {
        source,
        candidates,
        top_n,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    #[allow(non_upper_case_globals)]
    static source: LazyLock<PostData> = LazyLock::new(|| {
        PostData {
        title: "Deno.kill not working on windows".to_string(),
        content: r#"
Version: Deno 2.3.3
OS: Windows 11

Sending a SIGINT OS signal on windows like: Deno.kill(Deno.pid, 'SIGINT');
Results in: TypeError: Windows only supports ctrl-c (SIGINT) and ctrl-break (SIGBREAK), but got SIGINT
            "#.to_string(),
    }
    });

    #[allow(non_upper_case_globals)]
    static candidates: LazyLock<Vec<PostData>> = LazyLock::new(|| {
        vec![
            PostData {
                title: "Deno.kill on windows".to_string(),
            content: r#"
Version: Deno 2.3.3
OS: Windows 11

Sending a SIGINT OS signal on windows like: Deno.kill(Deno.pid, 'SIGINT');
Results in: TypeError: Windows only supports ctrl-c (SIGINT) and ctrl-break (SIGBREAK), but got SIGINT

Same goes for SIGBREAK

Registering event listeners with: Deno.addSignalListener('SIGINT', doSomething); Works correctly
            "#.to_string(),
            },
            PostData {
                title: "denojs on termux like nodejs".to_string(),
                content: r#"
We want a smooth download for Deno.js like Node.js, Python, etc., instead of downloading extra
libraries on Termux. We seek a streamlined download process for Deno.js similar to that of Node.js
and Python, rather than having to download additional libraries on Termux.
"#.to_string(),
            },
        ]
    });

    #[test]
    fn test_find_similar_posts_native() {
        let FindTopNResult { matches, .. } =
            find_similar_posts_native(source.clone(), candidates.clone(), 1).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].target.title, "Deno.kill on windows");
    }

    #[test]
    fn test_find_similar_posts_native_parallel() {
        let FindTopNResult { matches, .. } =
            find_similar_posts_native_parallel(source.clone(), candidates.clone(), 1).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].target.title, "Deno.kill on windows");
    }
}

use std::sync::{Arc, RwLock};

use napi::{bindgen_prelude::AsyncTask, Env, Error, Result, Task};

use crate::{do_find_similar_posts_native_parallel, FindTopNResult, PostData};

#[napi]
pub struct PostStore {
    posts: Arc<RwLock<Vec<PostData>>>,
}

#[napi]
impl PostStore {
    #[napi(constructor)]
    pub fn new() -> Self {
        PostStore {
            posts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    #[napi]
    pub fn preload(&self, posts: Vec<PostData>) -> Result<()> {
        match self.posts.write() {
            Ok(mut _posts) => {
                *_posts = posts;
                Ok(())
            }
            Err(e) => Err(Error::from_reason(format!(
                "Failed to preload posts: {}",
                e
            ))),
        }
    }

    #[napi(ts_return_type = "Promise<FindTopNResult>")]
    pub fn find_similar_posts(
        &self,
        source: PostData,
        top_n: u32,
    ) -> AsyncTask<AsyncFindSimilarPosts> {
        AsyncTask::new(AsyncFindSimilarPosts {
            source,
            posts: self.posts.clone(),
            top_n,
        })
    }
}

pub struct AsyncFindSimilarPosts {
    source: PostData,
    posts: Arc<RwLock<Vec<PostData>>>,
    top_n: u32,
}

#[napi]
impl Task for AsyncFindSimilarPosts {
    type Output = FindTopNResult;
    type JsValue = FindTopNResult;

    fn compute(&mut self) -> Result<Self::Output> {
        match self.posts.read() {
            Ok(posts) => do_find_similar_posts_native_parallel(&self.source, &posts, self.top_n),
            Err(e) => Err(Error::from_reason(format!("Failed to read posts: {}", e))),
        }
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

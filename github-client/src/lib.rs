use anyhow::bail;
use repos_response::Repositories;
use reqwest::Client;
use reqwest::StatusCode;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;

mod git_clone;
pub mod repos_response;

#[derive(Clone)]
pub struct GithubClient {
    clone_state: Arc<Mutex<()>>,
    client: Client,
}

impl GithubClient {
    pub fn into_ghc_trait(self) -> Arc<dyn GhcTrait + Send + Sync> {
        Arc::new(self)
    }
    pub fn as_ghc_trait(&self) -> Arc<dyn GhcTrait + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl Default for GithubClient {
    fn default() -> Self {
        Self {
            clone_state: Arc::new(Mutex::new(())),
            client: Client::builder()
                .user_agent("reqwest")
                .build()
                .expect("We want a client to be available"),
        }
    }
}
#[async_trait::async_trait]
impl GhcTrait for GithubClient {
    async fn get_repos(&self) -> anyhow::Result<Repositories> {
        let request = self
            .client
            .get("https://api.github.com/users/simon-an/repos")
            .build()?;
        let response = self.client.execute(request).await?;
        match response.status() {
            StatusCode::OK => Ok(response.json::<Repositories>().await?),
            status => {
                let error = response.text().await?;
                eprintln!("{} {}", status, error);
                bail!("{} {}", status, error)
            }
        }
    }

    async fn clone_repository(&self, url: Url) -> anyhow::Result<()> {
        let url2 = url.clone();
        let last_path = url2.to_file_path().unwrap();TODO
        let path = Path::new(&last_path);
        log::info!("cloning into folder: {}", path.to_str().expect("bla"));
        match git_clone::run(url, &path) {
            Ok(()) => {}
            Err(e) => log::error!("error: {}", e),
        }

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait GhcTrait {
    async fn get_repos(&self) -> anyhow::Result<Repositories>;
    async fn clone_repository(&self, url: Url) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::{repos_response::Repositories, GhcTrait};

    use super::GithubClient;

    #[tokio::test]
    async fn it_get_repos() {
        let client: GithubClient = Default::default();
        let repos = client.get_repos().await.unwrap();
        // println!("{:?}", repos);
        assert_eq!(repos.len(), 30);
    }

    #[test]
    fn deserialize() {
        let body = include_bytes!("output.json");
        let res: Repositories = serde_json::from_slice(body).unwrap();
        assert_eq!(res.len(), 30);
    }
}

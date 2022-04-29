#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use github_client::GithubClient;
use tracing::Instrument;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    use std::{sync::Arc, time::Duration};

    use github_client::GhcTrait;
    use tokio::sync::{mpsc, watch};

    let (tx, rx) = watch::channel(vec![]);
    let (clone_tx, mut clone_rx) = mpsc::channel(1);

    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_level(true)
        .with_target(false)
        .with_thread_ids(true)
        .init();

    let app = live_coding_12_2021::TemplateApp::new(rx.clone(), clone_tx);
    let native_options = eframe::NativeOptions::default();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(8)
        .thread_name("main-tokio")
        .build()
        .unwrap();

    let span_main = tracing::info_span!("main thread");
    let _guard = span_main.enter();
    let span = tracing::info_span!(parent: None, "poll repos thread");
    let client: GithubClient = Default::default();
    let client2 = client.clone();
    let _handle = rt.spawn(
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                let client = client.clone();
                interval.tick().await;
                log::info!("polling for repos");

                let client: Arc<dyn GhcTrait + Sync + Send> = client.into_ghc_trait();
                let repos = client.get_repos().await.unwrap();
                match tx.send(repos) {
                    Ok(_) => log::info!("sending result worked"),
                    Err(e) => log::error!("{e}"),
                }
            }
        }
        .instrument(span),
    );
  
    let span = tracing::info_span!(parent: None, "clone repo thread");
    let _handle = rt.spawn(
        async move {
            loop {
                let client = client2.clone();
                log::info!("waiting for clone command");
                let url: Option<String> = clone_rx.recv().await;
                log::info!("recieved clone command");
                if let Some(url) = url {
                    log::info!("cloning repo {url}");

                    let url = url.parse().expect("url string not an url");

                    let client: Arc<dyn GhcTrait + Sync + Send> = client.into_ghc_trait();
                    let _result = client.clone_repository(url).await.unwrap();
                    // match tx.send(repos) {
                    //     Ok(_) => log::info!("sending result worked"),
                    //     Err(e) => log::error!("{e}"),
                    // }
                }
            }
        }
        .instrument(span),
    );

    // let _result = handle.await.expect("Thread must join the main thread");

    eframe::run_native(Box::new(app), native_options);
}

use graphite::start_app;

#[tokio::main]
async fn main() {
    start_app().await.expect("failed to start the app");
}

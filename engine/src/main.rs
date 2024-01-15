use engine::app::{self, runApp};

#[tokio::main]
async fn main() {
    runApp().await.unwrap();
}

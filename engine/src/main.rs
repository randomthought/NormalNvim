use engine::app::{self, runApp};

#[actix_rt::main]
async fn main() {
    runApp().await.unwrap();
}

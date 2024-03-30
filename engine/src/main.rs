use engine::app::run_app;

#[actix_rt::main]
async fn main() {
    run_app().await.unwrap();
}

use data_forwarder::app::run_app;

#[actix::main]
async fn main() {
    run_app().await.unwrap();
}

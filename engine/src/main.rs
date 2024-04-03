use std::u64;

use engine::app::run_app;

#[actix_rt::main]
async fn main() {
    let data: u64 = 0;

    run_app().await.unwrap();
}

async fn happy(data: &u64) {
    todo!()
}

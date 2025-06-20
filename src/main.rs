mod select_m;

#[tokio::main]
async fn main() {
    select_m::start().await;
}

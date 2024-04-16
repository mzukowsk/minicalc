mod app;
mod functions;
mod sheet;

#[tokio::main]
async fn main() {
    match app::run().await {
        Ok(_) => println!("Finished successfully"),
        Err(e) => eprint!("Error: {:?}", e),
    }
}

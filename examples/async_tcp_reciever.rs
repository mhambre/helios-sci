use helios_sci::runtime::executor::Executor;

async fn start_server() {}

fn main() {
    let executor = Executor::new(start_server());
}

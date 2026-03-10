use helios_sci::sync::runtime::executor::Executor;

async fn start_server() {}

fn main() {
    let _executor = Executor::new(start_server());
}

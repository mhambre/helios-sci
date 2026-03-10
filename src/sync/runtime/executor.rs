//! The executor and task system for our asynchronous runtime.

pub struct Executor {}

impl Executor {
    pub fn new(_main_task: impl std::future::Future<Output = ()> + Send + 'static) -> Self {
        Self {}
    }
}

use keepr_core::Store;
use tokio::sync::{mpsc, oneshot};

type Job = Box<dyn FnOnce(&mut Store) + Send>;

#[derive(Clone)]
pub struct Database(mpsc::Sender<Job>);

impl Database {
    pub fn new(mut store: Store) -> Self {
        let (sender, mut receiver) = mpsc::channel::<Job>(64);
        std::thread::spawn(move || {
            while let Some(job) = receiver.blocking_recv() {
                job(&mut store);
            }
        });
        Self(sender)
    }

    pub async fn run<T, F>(&self, operation: F) -> Result<T, String>
    where
        T: Send + 'static,
        F: FnOnce(&mut Store) -> keepr_core::Result<T> + Send + 'static,
    {
        let (sender, receiver) = oneshot::channel();
        self.0
            .send(Box::new(move |store| {
                let result = operation(store).map_err(|error| {
                    tracing::error!(error = %error, "database operation failed");
                    error.code().to_owned()
                });
                let _ = sender.send(result);
            }))
            .await
            .map_err(|_| "storage_error".to_owned())?;
        receiver.await.map_err(|_| "storage_error".to_owned())?
    }
}

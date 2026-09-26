use ssh2::{Session, Sftp};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

use crate::{
    domain::{config_model::SftpConfig, error::AppError},
    sftp::connection::connect_blocking,
};

pub struct SessionPool {
    free: Mutex<Vec<Session>>,
    permits: Arc<Semaphore>,
    config: SftpConfig,
}

impl SessionPool {
    pub fn new(config: SftpConfig) -> Self {
        let size = config.pool_size;
        Self {
            free: Mutex::new(Vec::with_capacity(size)),
            permits: Arc::new(Semaphore::new(size)),
            config,
        }
    }

    pub async fn with_session<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Session) -> Result<T, AppError> + Send + 'static,
        T: Send + 'static,
    {
        // The permit bounds concurrency to pool_size, and is what makes the
        // free list a *pool* rather than an unbounded connection factory.
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|_| AppError::Internal)?;

        let session = match self.free.lock().await.pop() {
            Some(session) => session,
            None => connect_blocking(self.config.clone()).await?,
        };

        // The Session is moved into the blocking task and moved back out in
        // the return tuple, so every path -- Ok, Err, and even a panic inside
        // `f` -- gets the connection handled correctly with no manual return
        // call to forget.
        let handle = tokio::task::spawn_blocking(move || {
            let result = f(&session);
            (result, session)
        });

        match handle.await {
            Ok((result, session)) => {
                self.free.lock().await.push(session);
                result
            }
            // The task panicked, so `session` was dropped during unwind and
            // the connection is already closed. The next checkout reconnects.
            Err(e) => Err(AppError::SpawnBlocking(e)),
        }
    }
}

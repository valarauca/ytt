
use std::{
    future::{Ready,ready,Future},
    pin::Pin,
    sync::Arc,
    ops::{Deref,DerefMut},
};
use tokio::sync::{RwLock,Mutex};
use futures_util::future::{Either,FutureExt};

pub type MaybeFuture<O> = Either<Ready<O>,Pin<Box<dyn Future<Output=O> + 'static + Send>>>;

pub fn make_boxed<O,F>(arg: F) -> MaybeFuture<O>
where
    O: Send + 'static,
    F: Future<Output=O> + Send + 'static,
{
    let p: Pin<Box<dyn Future<Output=O> + 'static + Send>> = Box::pin(async move { arg.await });
    p.right_future()
}

trait MutexGuard<T: Send + Sync + 'static> {
    type ReadGuard: Deref<Target=T> + Send + 'static;
    type WriteGuard: DerefMut<Target=T> + Deref<Target=T> + Send + 'static;

    fn sync_read(self: Arc<Self>) -> Result<Self::ReadGuard,Arc<Self>>;
    fn async_read(self: Arc<Self>) -> impl Future<Output=Self::ReadGuard> + Send + 'static;
    fn sync_write(self: Arc<Self>) -> Result<Self::WriteGuard,Arc<Self>>;
    fn async_write(self: Arc<Self>) -> impl Future<Output=Self::WriteGuard> + Send + 'static;
}
impl<T: Send + Sync + 'static> MutexGuard<T> for RwLock<T> {
    type ReadGuard = tokio::sync::OwnedRwLockReadGuard<T>;
    type WriteGuard = tokio::sync::OwnedRwLockWriteGuard<T>;

    fn sync_read(self: Arc<Self>) -> Result<Self::ReadGuard,Arc<Self>> {
        self.clone().try_read_owned().map_err(|_| self)
    }

    fn async_read(self: Arc<Self>) -> impl Future<Output=Self::ReadGuard> + Send + 'static {
        async move { self.read_owned().await }
    }

    fn sync_write(self: Arc<Self>) -> Result<Self::WriteGuard,Arc<Self>> {
        self.clone().try_write_owned().map_err(|_| self)
    }

    fn async_write(self: Arc<Self>) -> impl Future<Output=Self::WriteGuard> + Send + 'static {
        async move { self.write_owned().await }
    }
}

#[allow(private_bounds)]
pub trait MaybeSyncAccess<T: Send + Sync + 'static>: MutexGuard<T> 
where
    Arc<Self>: Send + 'static,
{
    fn read<F,O>(self: Arc<Self>, lambda: F) -> MaybeFuture<O>
    where
        F: FnOnce(&T) -> O + Send + 'static,
        T: Send + Sync + 'static,
        O: Send + 'static,
    {
        match self.sync_read() {
            Ok(guard) => {
                ready((lambda)(&guard)).left_future()
            }
            Err(arc) => {
                let compat = move | arg: <Self as MutexGuard<T>>::ReadGuard | -> O { (lambda)(&arg) };
                // full type name b/c of rust-lang fun
                let arg: Pin<Box<dyn Future<Output=O> + 'static + Send>> = {
                    Box::pin(async move { 
                        let arc: Arc<Self> = arc;
                        arc.async_read().map(compat).await
                    })
                };
                arg.right_future()
            }
        }
    }

    fn write<F,O>(self: Arc<Self>, lambda: F) -> MaybeFuture<O>
    where
        F: FnOnce(&mut T) -> O + Send + 'static,
        T: Send + Sync + 'static,
        O: Send + 'static,
    { 
        match self.sync_write() {
            Ok(mut guard) => {
                ready((lambda)(&mut guard)).left_future()
            }
            Err(arc) => {
                let compat = move | mut arg: <Self as MutexGuard<T>>::WriteGuard | -> O { (lambda)(&mut arg) };
                // full type name b/c of rust-lang fun
                let arg: Pin<Box<dyn Future<Output=O> + 'static + Send>> = {
                    Box::pin(async move { 
                        let arc: Arc<Self> = arc;
                        arc.async_write().map(compat).await
                    })
                };
                arg.right_future()
            }
        }
    }
}
impl<T: Send + Sync + 'static, M: MutexGuard<T> + Send + Sync + 'static> MaybeSyncAccess<T> for M { }

pub trait MaybeErrAccess<T,E:>: Sized
where
    T: Send + Sync + 'static,
    E: Send + 'static,
{
    fn do_read<F,O>(self, lambda: F) -> MaybeFuture<Result<O,E>>
    where
        F: FnOnce(&T) -> Result<O,E> + Send + 'static,
        T: Send + Sync + 'static,
        O: Send + 'static;

    fn do_write<F,O>(self, lambda: F) -> MaybeFuture<Result<O,E>>
    where
        F: FnOnce(&mut T) -> Result<O,E> + Send + 'static,
        T: Send + Sync + 'static,
        O: Send + 'static;
}
impl<T,M,E> MaybeErrAccess<T,E> for Result<Arc<M>,E>
where
    T: Send + Sync + 'static,
    E: Send + 'static,
    M: MaybeSyncAccess<T> + MutexGuard<T>,
    Arc<M>: Send + 'static,
{
    fn do_read<F,O>(self, lambda: F) -> MaybeFuture<Result<O,E>>
    where
        F: FnOnce(&T) -> Result<O,E> + Send + 'static,
        O: Send + 'static
    {
        let x = match self {
            Ok(x) => x,
            Err(e) => return ready(Err(e)).left_future(),
        };
        x.read(lambda)
    }    
    fn do_write<F,O>(self, lambda: F) -> MaybeFuture<Result<O,E>>
    where
        F: FnOnce(&mut T) -> Result<O,E> + Send + 'static,
        O: Send + 'static
    {
        let x = match self {
            Ok(x) => x,
            Err(e) => return ready(Err(e)).left_future(),
        };
        x.write(lambda)
    }
}

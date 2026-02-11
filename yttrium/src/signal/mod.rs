use std::pin::Pin;
use futures_util::stream::Stream;

#[cfg(target_family = "unix")]
pub mod unix;
#[cfg(target_family = "unix")]
pub use self::unix::{signals};

#[cfg(target_family = "windows")]
pub mod win;
#[cfg(target_family = "windows")]
pub use self::win::{signals};


pub type OsAgnosticSignalStream = Pin<Box<dyn Stream<Item=SignalAction> + Send + 'static>>;

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Debug)]
#[repr(u8)]
pub enum SignalAction {
    Reload = 1,
    GracefulClose,
    Terminate,
}
impl SignalAction {
    pub const fn term(_:()) -> Self { Self::Terminate }
    pub const fn stop(_:()) -> Self { Self::GracefulClose }
    pub const fn reload(_: ()) -> Self { Self::Reload }
}


use std::{
    task::{Poll,Context},
};


use tokio::signal::windows::{CtrlBreak,CtrlC,CtrlClose,CtrlLogoff,CTrlShutdown};
use futures_util::stream::{select_all, StreamExt, Map, SelectAll};

use super::{SignalAction,OsAgnosticSignalStream};

pub fn signals() -> anyhow::Result<OsAgnosticSignalStream> {
    let signal_stream = build_iter([
        (SignalKind::Break, SignalAction::stop),
        (SignalKind::C, SignalAction::reload),
        (SignalKind::Close, SignalAction::term),
        (SignalKind::LogOff, SignalAction::term),
        (SignalKind::Shutdown, SignalAction::term),
    ])?;
    let stream: OsAgnosticSignalStream = Box::pin(signal_stream);
    Ok(stream)
}


type MappedSignalStream = Map<WinSignal,fn(()) -> SignalAction>;

fn build_stream(kind: SignalKind, arg: fn(()) -> SignalAction) -> anyhow::Result<MappedSignalStream> {
    Ok(WinSignal::new(kind).map(arg))
}

fn build_iter<const N: usize>(kinds: [(SignalKind,fn(()) -> SignalAction);N]) -> Result<SelectAll<MappedSignalStream>> {
    let mut signals = Vec::with_capacity(N);
    for item in kinds.into_iter().map(|(kind,lambda)| build_stream(kind,lambda)) {
        signals.push(item?);
    }
    Ok(select_all(signals))
}

/*
 * Isn't in tokio-stream for some reason, idk
 *
 */
enum SignalKind {
    Break,
    C,
    Close,
    LogOff,
    Shutdown,
}

enum WinSignal {
    Break(CtrlBreak),
    C(CtrlC),
    Close(CtrlClose),
    Logoff(CtrlLogoff),
    Shutdown(CtrlShutdown),
}
impl WinSignal {
    fn new(kind: SignalKind) -> anyhow::Result<Self> {
        match kind {
            SignalKind::Break => {
                Ok(WinSignal::Break(tokio::signal::windows::ctrl_break()?))
            }
            SignalKind::C => {
                Ok(WinSignal::Break(tokio::signal::windows::ctrl_c()?))
            }
            SignalKind::Close => {
                Ok(WinSignal::Break(tokio::signal::windows::ctrl_close()?))
            }
            SignalKind::Logoff => {
                Ok(WinSignal::Break(tokio::signal::windows::ctrl_logoff()?))
            }
            SignalKind::Shutdown => {
                Ok(WinSignal::Break(tokio::signal::windows::ctrl_shutdown()?))
            }
        }
    }
}
impl futures_util::stream::Stream for WinSignal {
    type Item = ();
    fn poll_next(self: Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            &mut Self::Break(ref mut x) => {
                let p: Pin<&mut CtrlBreak> = Pin::new(x);
                p.poll_next(ctx)
            }
            &mut Self::C(ref mut x) => {
                let p: Pin<&mut CtrlC> = Pin::new(x);
                p.poll_next(ctx)
            }
            &mut Self::Close(ref mut x) => {
                let p: Pin<&mut CtrlClose> = Pin::new(x);
                p.poll_next(ctx)
            }
            &mut Self::Logoff(ref mut x) => {
                let p: Pin<&mut CtrlLogoff> = Pin::new(x);
                p.poll_next(ctx)
            }
            &mut Self::Shutdown(ref mut x) => {
                let p: Pin<&mut CtrlShutdown> = Pin::new(x);
                p.poll_next(ctx)
            }
        }
    }
}


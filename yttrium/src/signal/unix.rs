use tokio::{
    signal::unix::{SignalKind,signal},
};
use tokio_stream::wrappers::SignalStream;
use futures_util::stream::{select_all, StreamExt, Map, SelectAll};

use super::{SignalAction,OsAgnosticSignalStream};


pub fn signals() -> anyhow::Result<OsAgnosticSignalStream> {
    let signal_stream = build_iter([
        (SignalKind::alarm(), SignalAction::stop),
        (SignalKind::hangup(), SignalAction::reload),
        (SignalKind::interrupt(), SignalAction::stop),
        (SignalKind::pipe(), SignalAction::stop),
        (SignalKind::terminate(), SignalAction::term),
        (SignalKind::quit(), SignalAction::term),
        (SignalKind::user_defined1(), SignalAction::stop),
        (SignalKind::user_defined2(), SignalAction::stop),
    ])?;
    let stream: OsAgnosticSignalStream = Box::pin(signal_stream);
    Ok(stream)
}

type MappedSignalStream = Map<SignalStream,fn(()) -> SignalAction>;

fn build_stream(kind: SignalKind, arg: fn(()) -> SignalAction) -> anyhow::Result<MappedSignalStream> {
    Ok(SignalStream::new(signal(kind)?).map(arg))
}

fn build_iter<const N: usize>(kinds: [(SignalKind,fn(()) -> SignalAction);N]) -> anyhow::Result<SelectAll<MappedSignalStream>> {
    let mut signals = Vec::with_capacity(N);
    for item in kinds.into_iter().map(|(kind,lambda)| build_stream(kind,lambda)) {
        signals.push(item?);
    }
    Ok(select_all(signals))
}

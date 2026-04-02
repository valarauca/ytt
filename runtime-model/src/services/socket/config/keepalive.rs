use socket2::{TcpKeepalive};
use serde::{Serialize,Deserialize};
use tracing::{warn};

use config_crap::{NiceDuration};

#[derive(Clone,PartialEq,Eq,Serialize,Deserialize,Debug)]
pub struct KeepAlive {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub time: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub interval: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub retries: Option<u32>,
}
impl KeepAlive {
    pub fn build(arg: &Option<Self>) -> Option<TcpKeepalive> {
        let arg: &Self = match arg {
            &Option::None => return None,
            &Option::Some(ref inner) => {
                if inner.time.is_none() && inner.interval.is_none() && inner.retries.is_none() {
                    return None;
                }
                inner
            }
        };
        let mut k = TcpKeepalive::new();

        k = match &arg.interval {
            &Option::None => k,
            &Option::Some(ref time) => k.with_time(time.get_duration()),
        };
        k = match &arg.interval {
            &Option::None => k,
            &Option::Some(ref time) => k.with_interval(time.get_duration()),
        };

        #[cfg(any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "ios",
                target_os = "visionos",
                target_os = "linux",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "tvos",
                target_os = "watchos",
            ))]
        fn retries(k: TcpKeepalive, arg: &u32) -> TcpKeepalive {
            k.with_retries(arg.clone())
        }
        #[cfg(not(any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "ios",
                target_os = "visionos",
                target_os = "linux",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "tvos",
                target_os = "watchos",
            )))]
        fn retries(k: TcpKeepalive, arg: &u32) -> TcpKeepalive {
            warn!("setting tcp keep alive tries is not support on this platform, value={:?} is ignored", arg);
            k
        }
        k = match &arg.retries {
            &Option::None => k,
            &Option::Some(ref count) => {
                retries(k, count)
            },
        };

        Some(k)
    }
}

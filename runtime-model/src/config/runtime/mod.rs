
use serde::{Serialize,Deserialize};

use tokio::runtime::{Runtime,Builder};

use config_crap::{NiceDuration};

#[derive(Clone,Serialize,Deserialize,PartialEq,Debug)]
pub enum Threads {
    #[serde(rename = "single")]
    Single,
    Multi {
        thread_stack_size: Option<usize>,
        thread_keep_alive: Option<NiceDuration>,
        max_blocking_threads: Option<usize>,
    }
}
impl Default for Threads {
    fn default() -> Self {
        Self::Multi { 
            thread_stack_size: None,
            thread_keep_alive: None,
            max_blocking_threads: None,
        }
    }
}
impl Threads {
    pub fn spawn(&self) -> Builder {
        match self {
            &Self::Single => Builder::new_current_thread(),
            &Self::Multi { ref thread_stack_size, ref thread_keep_alive, ref max_blocking_threads } => {
                let mut b = Builder::new_multi_thread();
                if let Option::Some(stack) = thread_stack_size {
                    b.thread_stack_size(*stack);
                }
                if let Option::Some(dur) = thread_keep_alive {
                    b.thread_keep_alive(dur.get_duration());
                }
                if let Option::Some(max_block) = max_blocking_threads {
                    b.max_blocking_threads(*max_block);
                }
                b.thread_name("yttrium-worker");
                b
            }
        }
    }
}

#[derive(Clone,Serialize,Deserialize,Default,PartialEq,Debug)]
pub struct TokioRuntime {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub event_interval: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub global_queue_interval: Option<u32>,
    #[serde(default)]
    pub threads: Threads,
}
impl TokioRuntime {
    pub fn build(&self) -> Result<Runtime,std::io::Error> {
        let mut b = self.threads.spawn();
        if let Option::Some(x) = self.event_interval {
            b.event_interval(x);
        }
        if let Option::Some(x) = self.global_queue_interval {
            b.global_queue_interval(x);
        }
        b.enable_all();
        b.build()
    }
}

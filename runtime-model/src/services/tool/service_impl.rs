use serde::{Deserialize,Serialize};
use url::Url;
use serde_json::{Value as JSValue};

use openrouter::{
    completions::{Response as ORResponse, Request as ORRequest},
};
use crate::{
    adapters::{
        s3service::{BoxCloneSyncService},
        maybe_async::{make_boxed,make_ready,MaybeFuture},
    }
};





pub enum ToolConfigServiceRequest {
    Info,
    Request {
        data: JSValue,
    }
}

pub enum ToolConfigServiceResponse {
    Info {
        default_name: Option<String>,
        default_descrption: Option<String>,
        validation: Option<JSValue>,
    }
}



enum ToolServiceInternal {
    OpenRouter {
        service: BoxCloneSyncService<ORRequest,ORResponse>,
    }
}

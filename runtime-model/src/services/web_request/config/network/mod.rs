use serde::{Deserialize,Serialize};
use reqwest::{ClientBuilder};

use crate::generic_config::client::traits::{Apply};

mod bind;
pub use self::bind::Bind;
mod tcp;
pub use self::tcp::Tcp;
mod pool;
pub use self::pool::Pool;
mod tls;
pub use self::tls::{Tls};
/*
mod connection;
pub use self::connection::{Connection};
*/

#[derive(Clone,Serialize,Deserialize,PartialEq,Eq,Debug,Reflect)]
pub struct Networking {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub socket: Option<Bind>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub tcp: Option<Tcp>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub pool: Option<Pool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub tls: Option<Tls>,
    /*
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub connection: Option<Connection>,
    */
}
impl Apply for Networking {
    fn apply_opts(&self, b: ClientBuilder) -> ClientBuilder {

        let b = Bind::apply(&self.socket, b);
        let b = Tcp::apply(&self.tcp, b);
        let b = Pool::apply(&self.pool, b);
        let b = Tls::apply(&self.tls, b);
        //let b = Connection::apply(&self.connection, b);

        b
    }
}

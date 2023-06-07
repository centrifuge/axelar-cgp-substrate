use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::server::Server;
use jsonrpsee::ws_client::WsClientBuilder;
use sp_runtime::{
    generic::SignedBlock,
    traits::{Block as BlockT, NumberFor},
};
use sc_rpc_api::{
    chain::ChainApiClient, author::AuthorApiClient
};

/// An abstraction that allows for consumers to easily
/// verify and fetch Axelar specific information from
/// any Substrate based chain that uses the
/// pallet-gateway provided by this project.
#[async_trait::async_trait]
trait SubstrateChain: Send + 'static {
    type GatewayConfig: axelar_cgp::Config;

    type Header;

    type Hash;

    fn heads(&self) -> Box<dyn futures::Stream<Item = Self::Hash>>;

    fn finalized_heads(&self) -> Box<dyn futures::Stream<Item = Self::Hash>>;

    async fn head(&self, block: Self::Hash) -> Self::Header;

    async fn latest_finalized_head(&self) -> Self::Hash;

    async fn finalized(&self, hash: Self::Hash) -> bool;

    async fn events_for(&self, hash: Self::Hash) -> axelar_cgp::Event<Self::GatewayConfig>;
}


#[async_trait::async_trait]
trait SubstrateBroadcaster: Send + 'static {
    type Extrinsic;

    async fn submit(&self, xt: Self::Extrinsic);
}

#[derive(Clone)]
pub struct Client<Block, Config> {
    inner: Arc<jsonrpsee::core::client::Client>,
    _phantom: PhantomData<(Block, Config)>
}

impl<Block, Config> Client<Block, Config> {
    pub async fn new(chain: impl AsRef<str>) -> Self {
        let inner = Arc::new(jsonrpsee::ws_client::WsClientBuilder::default().build(chain).await.expect("Could not connect to chain..."));

        Self {
            inner,
            _phantom: Default::default()
        }
    }
}

#[async_trait::async_trait]
impl<Block: BlockT + 'static, Config: axelar_cgp::Config + Send + Sync> SubstrateChain for Client<Block, Config> {
    type GatewayConfig = Config;

    type Header = Block::Header;

    type Hash = Block::Hash;

    fn heads(&self) -> Box<dyn futures::Stream<Item = Self::Hash>> {
        todo!()
    }

    fn finalized_heads(&self) -> Box<dyn futures::Stream<Item = Self::Hash>> {
        todo!()
    }

    async fn head(&self, block: Self::Hash) -> Self::Header {
        //ChainApiClient::<NumberFor<Block>, Block::Hash, Block::Header, SignedBlock<Block>>::header(&(*self.inner), Some(block())).unwrap()
        todo!()
    }

    async fn latest_finalized_head(&self) -> Self::Hash {
        //ChainApiClient::<NumberFor<Block>, Block::Hash, Block::Header, SignedBlock<Block>>::finalized_head(&(*self.inner)).unwrap()
        todo!()
    }

    async fn finalized(&self, hash: Self::Hash) -> bool{
        todo!()
    }

    async fn events_for(&self, hash: Self::Hash) -> axelar_cgp::Event<Self::GatewayConfig> {
        todo!()
    }
}
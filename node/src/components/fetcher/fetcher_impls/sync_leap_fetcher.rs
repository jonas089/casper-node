use std::{collections::HashMap, time::Duration};

use num_rational::Ratio;

use crate::{
    components::fetcher::{metrics::Metrics, Event, FetchResponder, Fetcher, ItemFetcher},
    effect::{requests::StorageRequest, EffectBuilder, EffectExt, Effects},
    types::{BlockHash, NodeId, SyncLeap},
};

impl ItemFetcher<SyncLeap> for Fetcher<SyncLeap> {
    // We want the fetcher to ask all the peers we give to it separately, and return their
    // responses separately, not just respond with the first SyncLeap it successfully gets from a
    // single peer.
    const SAFE_TO_RESPOND_TO_ALL: bool = false;

    fn responders(
        &mut self,
    ) -> &mut HashMap<BlockHash, HashMap<NodeId, Vec<FetchResponder<SyncLeap>>>> {
        &mut self.responders
    }

    fn validation_metadata(&self) -> &Ratio<u64> {
        &self.validation_metadata
    }

    fn metrics(&mut self) -> &Metrics {
        &self.metrics
    }

    fn peer_timeout(&self) -> Duration {
        self.get_from_peer_timeout
    }

    fn get_from_storage<REv>(
        &mut self,
        effect_builder: EffectBuilder<REv>,
        id: BlockHash,
        peer: NodeId,
        validation_metadata: Ratio<u64>,
        responder: FetchResponder<SyncLeap>,
    ) -> Effects<Event<SyncLeap>>
    where
        REv: From<StorageRequest> + Send,
    {
        effect_builder
            .immediately()
            .event(move |()| Event::GetFromStorageResult {
                id,
                peer,
                validation_metadata,
                maybe_item: Box::new(None),
                responder,
            })
    }

    fn put_to_storage<REv>(
        &self,
        _item: SyncLeap,
        _peer: NodeId,
        _effect_builder: EffectBuilder<REv>,
    ) -> Option<Effects<Event<SyncLeap>>>
    where
        REv: From<StorageRequest> + Send,
    {
        None
    }
}

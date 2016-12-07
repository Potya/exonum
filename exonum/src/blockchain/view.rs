use std::slice::SliceConcatExt;
use std::ops::Deref;

use ::crypto::{Hash, PublicKey};
use ::messages::{Precommit, Message, ConfigPropose, ConfigVote, AnyTx};
use ::storage::{StorageValue, Fork, ListTable, MapTable, MerkleTable, MerklePatriciaTable,
                HeightBytes, Error, Map, List};

use super::Block;

pub type ConfigurationData = Vec<u8>;

pub trait View<F: Fork>: Deref<Target = F> {
    type Transaction: Message + StorageValue;

    fn last_block(&self) -> Result<Option<Block>, Error> {
        Ok(match self.heights().last()? {
            Some(hash) => Some(self.blocks().get(&hash)?.unwrap()),
            None => None,
        })
    }

    fn from_fork(fork: F) -> Self;

    fn transactions(&self) -> MapTable<F, Hash, AnyTx<Self::Transaction>> {
        MapTable::new(vec![00], self)
    }

    fn blocks(&self) -> MapTable<F, Hash, Block> {
        MapTable::new(vec![01], self)
    }

    fn heights(&self) -> ListTable<MapTable<F, [u8], Vec<u8>>, u64, Hash> {
        ListTable::new(MapTable::new(vec![02], self))
    }

    fn block_txs(&self, height: u64) -> MerkleTable<MapTable<F, [u8], Vec<u8>>, u32, Hash> {
        let mut vecb = Vec::with_capacity(1 + height.len_hint()); 
        vecb.push(03u8); 
        vecb = height.serialize(vecb); 
        MerkleTable::new(MapTable::new(vecb, self))
    }

    fn precommits(&self, hash: &Hash) -> ListTable<MapTable<F, [u8], Vec<u8>>, u32, Precommit> {
        ListTable::new(MapTable::new([&[03], hash.as_ref()].concat(), self))
    }

    fn config_proposes(&self)
                       -> MerklePatriciaTable<MapTable<F, [u8], Vec<u8>>, Hash, ConfigPropose> {
        // config_propose paricia merkletree <hash_tx> транзакция пропоз
        MerklePatriciaTable::new(MapTable::new(vec![04], self))
    }

    fn config_votes(&self)
                    -> MerklePatriciaTable<MapTable<F, [u8], Vec<u8>>, PublicKey, ConfigVote> {
        // config_votes patricia merkletree <pub_key> последний голос
        MerklePatriciaTable::new(MapTable::new(vec![05], self))
    }

    fn configs
        (&self)
         -> MerklePatriciaTable<MapTable<F, [u8], Vec<u8>>, HeightBytes, ConfigurationData> {
        // configs patricia merkletree <высота блока> json
        MerklePatriciaTable::new(MapTable::new(vec![06], self))
    }

    // TODO: consider List index to reduce storage volume
    fn configs_heights(&self) -> ListTable<MapTable<F, [u8], Vec<u8>>, u64, HeightBytes> {
        ListTable::new(MapTable::new(vec![07], self))
    }
}

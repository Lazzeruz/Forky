use kaspa_consensus_core::BlockHasher;
use kaspa_database::prelude::StoreError;
use kaspa_database::prelude::DB;
use kaspa_database::prelude::{BatchDbWriter, CachedDbAccess, DirectDbWriter};
use kaspa_hashes::Hash;
use kaspa_math::Uint3072;
use kaspa_muhash::MuHash;
use rocksdb::WriteBatch;
use std::sync::Arc;

pub trait UtxoMultisetsStoreReader {
    fn get(&self, hash: Hash) -> Result<MuHash, StoreError>;
}

pub trait UtxoMultisetsStore: UtxoMultisetsStoreReader {
    fn insert(&self, hash: Hash, multiset: MuHash) -> Result<(), StoreError>;
}

const STORE_PREFIX: &[u8] = b"utxo-multisets";

/// A DB + cache implementation of `DbUtxoMultisetsStore` trait, with concurrency support.
#[derive(Clone)]
pub struct DbUtxoMultisetsStore {
    db: Arc<DB>,
    access: CachedDbAccess<Hash, Uint3072, BlockHasher>,
}

impl DbUtxoMultisetsStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self { db: Arc::clone(&db), access: CachedDbAccess::new(Arc::clone(&db), cache_size, STORE_PREFIX.to_vec()) }
    }

    pub fn clone_with_new_cache(&self, cache_size: u64) -> Self {
        Self::new(Arc::clone(&self.db), cache_size)
    }

    pub fn insert_batch(&self, batch: &mut WriteBatch, hash: Hash, multiset: MuHash) -> Result<(), StoreError> {
        if self.access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.access.write(BatchDbWriter::new(batch), hash, multiset.try_into().expect("multiset is expected to be finalized"))?;
        Ok(())
    }
}

impl UtxoMultisetsStoreReader for DbUtxoMultisetsStore {
    fn get(&self, hash: Hash) -> Result<MuHash, StoreError> {
        Ok(self.access.read(hash)?.into())
    }
}

impl UtxoMultisetsStore for DbUtxoMultisetsStore {
    fn insert(&self, hash: Hash, multiset: MuHash) -> Result<(), StoreError> {
        if self.access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.access.write(DirectDbWriter::new(&self.db), hash, multiset.try_into().expect("multiset is expected to be finalized"))?;
        Ok(())
    }
}

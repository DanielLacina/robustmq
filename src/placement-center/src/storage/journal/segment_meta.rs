// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{
    key_all_segment_metadata, key_segment_metadata, key_segment_metadata_cluster_prefix,
    key_segment_metadata_namespace_prefix, key_segment_metadata_shard_prefix,
};
use crate::storage::rocksdb::RocksDBEngine;

pub struct SegmentMetadataStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SegmentMetadataStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SegmentMetadataStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, segment: JournalSegmentMetadata) -> Result<(), CommonError> {
        let shard_key = key_segment_metadata(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
            segment.segment_seq,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), shard_key, segment)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<Option<JournalSegmentMetadata>, CommonError> {
        let shard_key: String =
            key_segment_metadata(cluster_name, namespace, shard_name, segment_seq);

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)? {
            return Ok(Some(serde_json::from_slice::<JournalSegmentMetadata>(
                &data.data,
            )?));
        }

        Ok(None)
    }

    pub fn all_segment(&self) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_all_segment_metadata();
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_slice::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_cluster(
        &self,
        cluster_name: &str,
    ) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_segment_metadata_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_slice::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_namespace(
        &self,
        cluster_name: &str,
        namespace: &str,
    ) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_segment_metadata_namespace_prefix(cluster_name, namespace);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_slice::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_shard(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<JournalSegmentMetadata>, CommonError> {
        let prefix_key = key_segment_metadata_shard_prefix(cluster_name, namespace, shard_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_slice::<JournalSegmentMetadata>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<(), CommonError> {
        let shard_key = key_segment_metadata(cluster_name, namespace, shard_name, segment_seq);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)
    }
}

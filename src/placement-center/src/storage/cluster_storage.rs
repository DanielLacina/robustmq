use super::rocksdb::RocksDBStorage;
use common::log::error_meta;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct ClusterStorage {
    rds: Arc<RocksDBStorage>,
}

impl ClusterStorage {
    pub fn new(rds: Arc<RocksDBStorage>) -> Self {
        ClusterStorage { rds }
    }

    // get node list
    pub fn node_list(&self, cluster_name: String) -> Vec<NodeInfo> {
        let mut result = Vec::new();
        let cf = self.rds.cf_cluster();
        let cluster_info = self.get_cluster(&cluster_name);
        if cluster_info.is_none() {
            return result;
        }
        for node_id in cluster_info.unwrap().nodes {
            let key = self.node_key(&cluster_name, node_id);
            match self.rds.read::<NodeInfo>(cf, &key) {
                Ok(node_info) => {
                    if let Some(ni) = node_info {
                        result.push(ni);
                    }
                }
                Err(_) => {}
            };
        }
        return result;
    }

    // save node info
    pub fn save_node(&self, cluster_name: String, cluster_type: String, node: NodeInfo) {
        let cf = self.rds.cf_cluster();

        // save or update cluster info
        let mut ci = ClusterInfo::default();

        let mut cluster_info = self.get_cluster(&cluster_name);
        if cluster_info.is_none() {
            ci.cluster_name = cluster_name.clone();
        } else {
            ci = cluster_info.unwrap();
        }

        ci.nodes.push(node.node_id);
        ci.nodes.dedup();
        self.save_cluster(ci);

        // save node info
        let node_key = self.node_key(&cluster_name, node.node_id);
        match self.rds.write(cf, &node_key, &node) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn remove_node(&self, cluster_name: String, node_id: u64) {
        let cf = self.rds.cf_cluster();

        // save or update cluster info
        let mut cluster_info = self.get_cluster(&cluster_name);
        if !cluster_info.is_none() {
            let mut ci = cluster_info.unwrap();
            let mut nodes = Vec::new();
            for nid in ci.nodes {
                if nid != node_id {
                    nodes.push(nid);
                }
            }
            ci.nodes = nodes;
            self.save_cluster(ci);
        }

        // delete node info
        let node_key = self.node_key(&cluster_name, node_id);
        match self.rds.delete(cf, &node_key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    // save cluster info
    pub fn save_cluster(&self, cluster_info: ClusterInfo) {
        let cf = self.rds.cf_cluster();
        let cluster_key = self.cluster_key(&cluster_info.cluster_name);
        match self.rds.write(cf, &cluster_key, &cluster_info) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    // get cluster info
    pub fn get_cluster(&self, cluster_name: &String) -> Option<ClusterInfo> {
        let cf = self.rds.cf_cluster();
        let cluster_key = self.cluster_key(&cluster_name);
        match self.rds.read::<ClusterInfo>(cf, &cluster_key) {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }

    // get node info
    pub fn get_node(&self, cluster_name: String, node_id: u64) -> Option<NodeInfo> {
        let cf = self.rds.cf_cluster();
        let cluster_key = self.node_key(&cluster_name, node_id);
        match self.rds.read::<NodeInfo>(cf, &cluster_key) {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }
}

impl ClusterStorage {
    fn node_key(&self, cluster_name: &String, node_id: u64) -> String {
        return format!("node_{}_{}", cluster_name, node_id);
    }

    fn cluster_key(&self, cluster_name: &String) -> String {
        return format!("node_{}", cluster_name);
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster_name: String,
    pub nodes: Vec<u64>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: u64,
    pub node_ip: String,
    pub node_port: u32,
}

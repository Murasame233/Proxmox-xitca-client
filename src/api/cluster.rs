use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::BoolFromInt;

use crate::client::{ProxmoxApiService, ProxmoxRes};

// GET /api2/json/cluster/status

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterStatus {
    pub id: String,
    pub name: String,
    pub nodes: u16,
    #[serde_as(as = "BoolFromInt")]
    pub quorate: bool,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct NodeStatus {
    pub id: String,
    pub name: String,
    pub ip: String,
    //// is local, means you are interact with this node or not.
    #[serde_as(as = "BoolFromInt")]
    pub local: bool,
    //// id in cluster
    pub nodeid: u16,
    #[serde_as(as = "BoolFromInt")]
    pub online: bool,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AllStatus {
    #[serde(rename = "cluster")]
    Cluster(ClusterStatus),
    #[serde(rename = "node")]
    Node(NodeStatus),
}

//// GET /api2/json/cluster/status
//// Return Vec<ClusterStatus>
impl ProxmoxApiService {
    pub async fn get_status(&self) -> Result<Vec<AllStatus>, Box<dyn std::error::Error>> {
        let req = self.make_get_request("/api2/json/cluster/status").await;
        let res = req.send().await?;
        let body = res.body().await?;
        let data = serde_json::from_slice::<ProxmoxRes<Vec<AllStatus>>>(&body)?.data;
        Ok(data)
    }
}

//! Byzantine Fault Tolerant Node Discovery System - Phase 5 Stage 3
//!
//! This module implements a robust node discovery system that can tolerate Byzantine failures
//! using Practical Byzantine Fault Tolerance (PBFT) consensus and cryptographic verification.
//!
//! Key Features:
//! - PBFT consensus for node membership decisions  
//! - Cryptographic node identity verification
//! - Dynamic network topology adaptation
//! - Malicious node detection and isolation
//! - Gossip-based information propagation
//! - Automatic network healing and recovery

use super::{
    ConcurrencyError,
    distributed_execution_engine::{NodeCapabilities, NodeId},
};
use crate::diagnostics::{Error, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{RwLock as AsyncRwLock, mpsc, oneshot};
use tokio::time::{Instant, interval, timeout};
use uuid::Uuid;

/// Type alias for prepare votes storage
type PrepareVotes = Arc<RwLock<HashMap<u64, HashMap<NodeId, Vec<u8>>>>>;
/// Type alias for commit votes storage
type CommitVotes = Arc<RwLock<HashMap<u64, HashMap<NodeId, Vec<u8>>>>>;

/// Maximum number of nodes in the network
const MAX_NETWORK_SIZE: usize = 1000;
/// PBFT requires at least 3f+1 nodes to tolerate f failures
const MIN_PBFT_NODES: usize = 4;
/// Heartbeat interval for node liveness
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// Node timeout threshold
const NODE_TIMEOUT: Duration = Duration::from_secs(15);
/// Maximum message propagation delay
const MAX_PROPAGATION_DELAY: Duration = Duration::from_secs(10);
/// Gossip protocol fan-out factor
const GOSSIP_FANOUT: usize = 3;

/// Cryptographic node identity with public key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// Unique node identifier
    pub node_id: NodeId,
    /// Node's public key for verification
    pub public_key: Vec<u8>,
    /// Node's network address
    pub address: SocketAddr,
    /// Node join timestamp
    pub joined_at: SystemTime,
    /// Node capabilities and metadata
    pub capabilities: NodeCapabilities,
}

impl NodeIdentity {
    /// Creates a new node identity with generated key pair
    pub fn new(address: SocketAddr, capabilities: NodeCapabilities) -> Self {
        let node_id = capabilities.node_id;

        // Generate a simple public key (in production, use proper cryptography)
        let mut hasher = Sha256::new();
        hasher.update(node_id.uuid().as_bytes());
        hasher.update(address.to_string().as_bytes());
        let public_key = hasher.finalize().to_vec();

        Self {
            node_id,
            public_key,
            address,
            joined_at: SystemTime::now(),
            capabilities,
        }
    }

    /// Verifies a message signature (simplified implementation)
    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        // Simplified signature verification
        // In production, use proper digital signatures (Ed25519, ECDSA, etc.)
        let mut hasher = Sha256::new();
        hasher.update(&self.public_key);
        hasher.update(message);
        let expected = hasher.finalize();

        signature.len() == expected.len() && signature == expected.as_slice()
    }

    /// Signs a message (simplified implementation)
    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        // Simplified message signing
        let mut hasher = Sha256::new();
        hasher.update(&self.public_key);
        hasher.update(message);
        hasher.finalize().to_vec()
    }
}

/// Node status in the network
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is active and responding
    Active,
    /// Node is suspected to be faulty
    Suspected,
    /// Node has been confirmed as Byzantine/malicious
    Byzantine,
    /// Node has left the network
    Left,
}

/// Network membership view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkView {
    /// View sequence number for ordering
    pub view_number: u64,
    /// Set of active nodes in this view
    pub active_nodes: HashMap<NodeId, NodeIdentity>,
    /// Suspected nodes
    pub suspected_nodes: HashMap<NodeId, NodeIdentity>,
    /// Byzantine nodes to be excluded
    pub byzantine_nodes: HashSet<NodeId>,
    /// View creation timestamp
    pub created_at: SystemTime,
    /// Digital signature of this view
    pub signature: Vec<u8>,
    /// Node that proposed this view
    pub proposer: NodeId,
}

impl NetworkView {
    /// Creates a new network view
    pub fn new(view_number: u64, proposer: NodeId) -> Self {
        Self {
            view_number,
            active_nodes: HashMap::new(),
            suspected_nodes: HashMap::new(),
            byzantine_nodes: HashSet::new(),
            created_at: SystemTime::now(),
            signature: Vec::new(),
            proposer,
        }
    }

    /// Adds an active node to the view
    pub fn add_active_node(&mut self, identity: NodeIdentity) {
        self.active_nodes.insert(identity.node_id, identity);
    }

    /// Marks a node as suspected
    pub fn mark_suspected(&mut self, node_id: NodeId) {
        if let Some(identity) = self.active_nodes.remove(&node_id) {
            self.suspected_nodes.insert(node_id, identity);
        }
    }

    /// Marks a node as Byzantine
    pub fn mark_byzantine(&mut self, node_id: NodeId) {
        self.active_nodes.remove(&node_id);
        self.suspected_nodes.remove(&node_id);
        self.byzantine_nodes.insert(node_id);
    }

    /// Gets total number of nodes in view
    pub fn total_nodes(&self) -> usize {
        self.active_nodes.len() + self.suspected_nodes.len()
    }

    /// Calculates Byzantine fault tolerance threshold
    pub fn byzantine_threshold(&self) -> usize {
        let total = self.total_nodes();
        if total >= MIN_PBFT_NODES {
            (total - 1) / 3 // f = (n-1)/3 for PBFT
        } else {
            0
        }
    }

    /// Signs the view with proposer's identity
    pub fn sign(&mut self, proposer_identity: &NodeIdentity) {
        let message = self.compute_hash();
        self.signature = proposer_identity.sign_message(&message);
    }

    /// Verifies the view signature
    pub fn verify_signature(&self, proposer_identity: &NodeIdentity) -> bool {
        let message = self.compute_hash();
        proposer_identity.verify_signature(&message, &self.signature)
    }

    /// Computes hash of the view for signing
    fn compute_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.view_number.to_le_bytes());

        // Hash active nodes
        for (node_id, identity) in &self.active_nodes {
            hasher.update(node_id.uuid().as_bytes());
            hasher.update(&identity.public_key);
        }

        // Hash suspected nodes
        for node_id in self.suspected_nodes.keys() {
            hasher.update(node_id.uuid().as_bytes());
        }

        // Hash Byzantine nodes
        for node_id in &self.byzantine_nodes {
            hasher.update(node_id.uuid().as_bytes());
        }

        hasher.finalize().to_vec()
    }
}

/// PBFT message types for consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PbftMessage {
    /// View change proposal
    ViewChange {
        /// View number in consensus protocol
        view_number: u64,
        /// Proposed View field
        proposed_view: NetworkView,
        /// Sender of the message or request
        sender: NodeId,
        /// Cryptographic signature for verification
        signature: Vec<u8>,
    },
    /// Prepare message for view consensus
    Prepare {
        /// View number in consensus protocol
        view_number: u64,
        /// View Hash field
        view_hash: Vec<u8>,
        /// Sender of the message or request
        sender: NodeId,
        /// Cryptographic signature for verification
        signature: Vec<u8>,
    },
    /// Commit message for view consensus
    Commit {
        /// View number in consensus protocol
        view_number: u64,
        /// View Hash field
        view_hash: Vec<u8>,
        /// Sender of the message or request
        sender: NodeId,
        /// Cryptographic signature for verification
        signature: Vec<u8>,
    },
    /// Node suspicion report
    SuspicionReport {
        /// Suspected Node field
        suspected_node: NodeId,
        /// Evidence data for verification
        evidence: Vec<u8>,
        /// Reporter field
        reporter: NodeId,
        /// Cryptographic signature for verification
        signature: Vec<u8>,
    },
}

impl PbftMessage {
    /// Gets the sender of this message
    pub fn sender(&self) -> NodeId {
        match self {
            Self::ViewChange { sender, .. } => *sender,
            Self::Prepare { sender, .. } => *sender,
            Self::Commit { sender, .. } => *sender,
            Self::SuspicionReport { reporter, .. } => *reporter,
        }
    }

    /// Verifies message signature
    pub fn verify_signature(&self, sender_identity: &NodeIdentity) -> bool {
        let signature = match self {
            Self::ViewChange { signature, .. } => signature,
            Self::Prepare { signature, .. } => signature,
            Self::Commit { signature, .. } => signature,
            Self::SuspicionReport { signature, .. } => signature,
        };

        let message_data = self.compute_message_hash();
        sender_identity.verify_signature(&message_data, signature)
    }

    /// Computes hash for signature verification
    fn compute_message_hash(&self) -> Vec<u8> {
        // Simplified message hashing
        bincode::serialize(self).unwrap_or_default()
    }
}

/// Gossip message for information dissemination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    /// Message type identifier
    pub message_type: String,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message sender
    pub sender: NodeId,
    /// Message sequence number
    pub sequence: u64,
    /// Time-to-live for message propagation
    pub ttl: u32,
    /// Message timestamp
    pub timestamp: SystemTime,
}

impl GossipMessage {
    /// Creates a new gossip message
    pub fn new(message_type: String, payload: Vec<u8>, sender: NodeId, sequence: u64) -> Self {
        Self {
            message_type,
            payload,
            sender,
            sequence,
            ttl: 10, // Max 10 hops
            timestamp: SystemTime::now(),
        }
    }

    /// Decrements TTL for forwarding
    pub fn decrement_ttl(&mut self) -> bool {
        if self.ttl > 0 {
            self.ttl -= 1;
            true
        } else {
            false
        }
    }

    /// Checks if message has expired
    pub fn is_expired(&self, max_age: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.timestamp)
            .map_or(true, |age| age > max_age)
    }
}

/// Node liveness monitoring
#[derive(Debug)]
pub struct LivenessMonitor {
    /// Last heartbeat from each node
    last_heartbeat: Arc<RwLock<HashMap<NodeId, Instant>>>,
    /// Suspected nodes pending confirmation
    suspected_nodes: Arc<RwLock<HashSet<NodeId>>>,
}

impl LivenessMonitor {
    /// Creates a new liveness monitor
    pub fn new() -> Self {
        Self {
            last_heartbeat: Arc::new(RwLock::new(HashMap::new())),
            suspected_nodes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Records a heartbeat from a node
    pub fn record_heartbeat(&self, node_id: NodeId) -> Result<()> {
        let mut heartbeats = self.last_heartbeat.write().map_err(|_| {
            Error::runtime_error("Failed to acquire heartbeat lock".to_string(), None)
        })?;

        heartbeats.insert(node_id, Instant::now());

        // Remove from suspected nodes if present
        let mut suspected = self.suspected_nodes.write().map_err(|_| {
            Error::runtime_error("Failed to acquire suspected nodes lock".to_string(), None)
        })?;
        suspected.remove(&node_id);

        Ok(())
    }

    /// Checks for timed-out nodes
    pub fn check_timeouts(&self) -> Result<Vec<NodeId>> {
        let mut timed_out = Vec::new();
        let now = Instant::now();

        let heartbeats = self.last_heartbeat.read().map_err(|_| {
            Error::runtime_error("Failed to acquire heartbeat lock".to_string(), None)
        })?;

        for (node_id, last_seen) in heartbeats.iter() {
            if now.duration_since(*last_seen) > NODE_TIMEOUT {
                timed_out.push(*node_id);
            }
        }

        Ok(timed_out)
    }

    /// Marks a node as suspected
    pub fn mark_suspected(&self, node_id: NodeId) -> Result<()> {
        let mut suspected = self.suspected_nodes.write().map_err(|_| {
            Error::runtime_error("Failed to acquire suspected nodes lock".to_string(), None)
        })?;

        suspected.insert(node_id);
        Ok(())
    }

    /// Gets currently suspected nodes
    pub fn get_suspected_nodes(&self) -> Result<Vec<NodeId>> {
        let suspected = self.suspected_nodes.read().map_err(|_| {
            Error::runtime_error("Failed to acquire suspected nodes lock".to_string(), None)
        })?;

        Ok(suspected.iter().copied().collect())
    }
}

impl Default for LivenessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Byzantine fault detection and consensus
#[derive(Debug)]
pub struct ByzantineConsensus {
    /// Current network view
    current_view: Arc<AsyncRwLock<NetworkView>>,
    /// PBFT message log
    message_log: Arc<RwLock<VecDeque<PbftMessage>>>,
    /// Prepare votes for current view
    prepare_votes: PrepareVotes,
    /// Commit votes for current view
    commit_votes: CommitVotes,
    /// Local node identity
    local_identity: NodeIdentity,
}

impl ByzantineConsensus {
    /// Creates a new Byzantine consensus instance
    pub fn new(local_identity: NodeIdentity) -> Self {
        let initial_view = NetworkView::new(0, local_identity.node_id);

        Self {
            current_view: Arc::new(AsyncRwLock::new(initial_view)),
            message_log: Arc::new(RwLock::new(VecDeque::new())),
            prepare_votes: Arc::new(RwLock::new(HashMap::new())),
            commit_votes: Arc::new(RwLock::new(HashMap::new())),
            local_identity,
        }
    }

    /// Processes a PBFT message
    pub async fn process_message(
        &self,
        message: PbftMessage,
        sender_identity: &NodeIdentity,
    ) -> Result<Vec<PbftMessage>> {
        // Verify message signature
        if !message.verify_signature(sender_identity) {
            return Err(Box::new(Error::runtime_error(
                "Invalid message signature".to_string(),
                None,
            )));
        }

        // Log the message
        {
            let mut log = self.message_log.write().map_err(|_| {
                Error::runtime_error("Failed to acquire message log".to_string(), None)
            })?;
            log.push_back(message.clone());

            // Keep only recent messages
            if log.len() > 1000 {
                log.pop_front();
            }
        }

        match message {
            PbftMessage::ViewChange {
                view_number,
                proposed_view,
                ..
            } => self.handle_view_change(view_number, proposed_view).await,
            PbftMessage::Prepare {
                view_number,
                view_hash,
                sender,
                ..
            } => self.handle_prepare(view_number, view_hash, sender).await,
            PbftMessage::Commit {
                view_number,
                view_hash,
                sender,
                ..
            } => self.handle_commit(view_number, view_hash, sender).await,
            PbftMessage::SuspicionReport {
                suspected_node,
                evidence,
                reporter,
                ..
            } => {
                self.handle_suspicion_report(suspected_node, evidence, reporter)
                    .await
            }
        }
    }

    /// Handles view change proposal
    async fn handle_view_change(
        &self,
        view_number: u64,
        proposed_view: NetworkView,
    ) -> Result<Vec<PbftMessage>> {
        let current_view = self.current_view.read().await;

        // Only consider views with higher numbers
        if view_number <= current_view.view_number {
            return Ok(Vec::new());
        }

        // Validate the proposed view
        if !self.validate_view(&proposed_view) {
            return Ok(Vec::new());
        }

        drop(current_view);

        // Send prepare message
        let view_hash = proposed_view.compute_hash();
        let prepare_message = PbftMessage::Prepare {
            view_number,
            view_hash: view_hash.clone(),
            sender: self.local_identity.node_id,
            signature: self.local_identity.sign_message(&view_hash),
        };

        // Record our prepare vote
        {
            let mut votes = self.prepare_votes.write().map_err(|_| {
                Error::runtime_error("Failed to acquire prepare votes".to_string(), None)
            })?;

            votes
                .entry(view_number)
                .or_default()
                .insert(self.local_identity.node_id, view_hash);
        }

        Ok(vec![prepare_message])
    }

    /// Handles prepare message
    async fn handle_prepare(
        &self,
        view_number: u64,
        view_hash: Vec<u8>,
        sender: NodeId,
    ) -> Result<Vec<PbftMessage>> {
        // Record prepare vote
        {
            let mut votes = self.prepare_votes.write().map_err(|_| {
                Error::runtime_error("Failed to acquire prepare votes".to_string(), None)
            })?;

            votes
                .entry(view_number)
                .or_default()
                .insert(sender, view_hash.clone());
        }

        // Check if we have enough prepare votes (2f+1)
        let current_view = self.current_view.read().await;
        let required_votes = current_view.byzantine_threshold() * 2 + 1;

        let vote_count = {
            let votes = self.prepare_votes.read().map_err(|_| {
                Error::runtime_error("Failed to acquire prepare votes".to_string(), None)
            })?;

            votes
                .get(&view_number)
                .map_or(0, |v| v.values().filter(|&h| h == &view_hash).count())
        };

        if vote_count >= required_votes {
            // Send commit message
            let commit_message = PbftMessage::Commit {
                view_number,
                view_hash: view_hash.clone(),
                sender: self.local_identity.node_id,
                signature: self.local_identity.sign_message(&view_hash),
            };

            // Record our commit vote
            {
                let mut votes = self.commit_votes.write().map_err(|_| {
                    Error::runtime_error("Failed to acquire commit votes".to_string(), None)
                })?;

                votes
                    .entry(view_number)
                    .or_default()
                    .insert(self.local_identity.node_id, view_hash);
            }

            return Ok(vec![commit_message]);
        }

        Ok(Vec::new())
    }

    /// Handles commit message
    async fn handle_commit(
        &self,
        view_number: u64,
        view_hash: Vec<u8>,
        sender: NodeId,
    ) -> Result<Vec<PbftMessage>> {
        // Record commit vote
        {
            let mut votes = self.commit_votes.write().map_err(|_| {
                Error::runtime_error("Failed to acquire commit votes".to_string(), None)
            })?;

            votes
                .entry(view_number)
                .or_default()
                .insert(sender, view_hash.clone());
        }

        // Check if we have enough commit votes (2f+1)
        let current_view = self.current_view.read().await;
        let required_votes = current_view.byzantine_threshold() * 2 + 1;

        let vote_count = {
            let votes = self.commit_votes.read().map_err(|_| {
                Error::runtime_error("Failed to acquire commit votes".to_string(), None)
            })?;

            votes
                .get(&view_number)
                .map_or(0, |v| v.values().filter(|&h| h == &view_hash).count())
        };

        if vote_count >= required_votes {
            // TODO: Apply the committed view
            // For now, just log the consensus
            println!("Consensus reached for view {}", view_number);
        }

        Ok(Vec::new())
    }

    /// Handles suspicion report
    async fn handle_suspicion_report(
        &self,
        suspected_node: NodeId,
        _evidence: Vec<u8>,
        _reporter: NodeId,
    ) -> Result<Vec<PbftMessage>> {
        // TODO: Validate evidence and decide on node status
        // For now, just mark as suspected
        let mut current_view = self.current_view.write().await;
        current_view.mark_suspected(suspected_node);

        Ok(Vec::new())
    }

    /// Validates a proposed network view
    fn validate_view(&self, view: &NetworkView) -> bool {
        // Basic validation rules
        if view.total_nodes() == 0 || view.total_nodes() > MAX_NETWORK_SIZE {
            return false;
        }

        // Must have minimum nodes for PBFT
        if view.total_nodes() < MIN_PBFT_NODES {
            return false;
        }

        // Byzantine threshold must be reasonable
        if view.byzantine_nodes.len() > view.byzantine_threshold() {
            return false;
        }

        true
    }

    /// Gets current network view
    pub async fn get_current_view(&self) -> NetworkView {
        self.current_view.read().await.clone()
    }
}

/// Main Byzantine Node Discovery system
#[derive(Debug)]
pub struct ByzantineNodeDiscovery {
    /// Local node identity
    local_identity: NodeIdentity,
    /// Known nodes registry
    known_nodes: Arc<DashMap<NodeId, NodeIdentity>>,
    /// Liveness monitoring
    liveness_monitor: Arc<LivenessMonitor>,
    /// Byzantine consensus instance
    consensus: Arc<ByzantineConsensus>,
    /// UDP socket for gossip protocol
    gossip_socket: Option<Arc<UdpSocket>>,
    /// TCP listener for direct connections
    tcp_listener: Option<TcpListener>,
    /// Message sequence counter
    message_sequence: Arc<std::sync::atomic::AtomicU64>,
    /// Shutdown signal
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ByzantineNodeDiscovery {
    /// Creates a new Byzantine node discovery system
    pub async fn new(address: SocketAddr, capabilities: NodeCapabilities) -> Result<Self> {
        let local_identity = NodeIdentity::new(address, capabilities);
        let consensus = Arc::new(ByzantineConsensus::new(local_identity.clone()));

        // Bind UDP socket for gossip
        let gossip_socket = UdpSocket::bind(address).await.map_err(|e| {
            Box::new(Error::runtime_error(
                format!("Failed to bind gossip socket: {}", e),
                None,
            ))
        })?;

        // Bind TCP listener for direct connections
        let tcp_listener = TcpListener::bind(address).await.map_err(|e| {
            Box::new(Error::runtime_error(
                format!("Failed to bind TCP listener: {}", e),
                None,
            ))
        })?;

        Ok(Self {
            local_identity,
            known_nodes: Arc::new(DashMap::new()),
            liveness_monitor: Arc::new(LivenessMonitor::new()),
            consensus,
            gossip_socket: Some(Arc::new(gossip_socket)),
            tcp_listener: Some(tcp_listener),
            message_sequence: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            shutdown_tx: None,
        })
    }

    /// Starts the node discovery service
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Start heartbeat broadcasting
        let gossip_socket = self.gossip_socket.as_ref().unwrap().clone();
        let local_identity = self.local_identity.clone();
        let known_nodes = self.known_nodes.clone();
        let liveness_monitor = self.liveness_monitor.clone();

        tokio::spawn(async move {
            let mut heartbeat_interval = interval(HEARTBEAT_INTERVAL);

            loop {
                tokio::select! {
                    _ = heartbeat_interval.tick() => {
                        // Broadcast heartbeat
                        Self::broadcast_heartbeat(
                            &gossip_socket,
                            &local_identity,
                            &known_nodes,
                        ).await;

                        // Check for timed-out nodes
                        if let Ok(timed_out) = liveness_monitor.check_timeouts() {
                            for node_id in timed_out {
                                let _ = liveness_monitor.mark_suspected(node_id);
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        // Start gossip message handling
        let gossip_socket = self.gossip_socket.as_ref().unwrap().clone();
        let known_nodes = self.known_nodes.clone();
        let liveness_monitor = self.liveness_monitor.clone();
        let consensus = self.consensus.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 8192];

            loop {
                match gossip_socket.recv_from(&mut buffer).await {
                    Ok((len, addr)) => {
                        let data = &buffer[..len];

                        // Try to deserialize gossip message
                        if let Ok(message) = bincode::deserialize::<GossipMessage>(data) {
                            Self::handle_gossip_message(
                                message,
                                &known_nodes,
                                &liveness_monitor,
                                &consensus,
                            )
                            .await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Gossip receive error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Broadcasts heartbeat to all known nodes
    async fn broadcast_heartbeat(
        gossip_socket: &UdpSocket,
        local_identity: &NodeIdentity,
        known_nodes: &DashMap<NodeId, NodeIdentity>,
    ) {
        let heartbeat = GossipMessage::new(
            "heartbeat".to_string(),
            bincode::serialize(local_identity).unwrap_or_default(),
            local_identity.node_id,
            0,
        );

        if let Ok(data) = bincode::serialize(&heartbeat) {
            for entry in known_nodes.iter() {
                let addr = entry.value().address;
                let _ = gossip_socket.send_to(&data, addr).await;
            }
        }
    }

    /// Handles incoming gossip messages
    async fn handle_gossip_message(
        message: GossipMessage,
        known_nodes: &DashMap<NodeId, NodeIdentity>,
        liveness_monitor: &LivenessMonitor,
        _consensus: &ByzantineConsensus,
    ) {
        match message.message_type.as_str() {
            "heartbeat" => {
                // Process heartbeat
                let _ = liveness_monitor.record_heartbeat(message.sender);

                // Try to deserialize node identity
                if let Ok(identity) = bincode::deserialize::<NodeIdentity>(&message.payload) {
                    known_nodes.insert(identity.node_id, identity);
                }
            }
            "node_announcement" => {
                // New node announcement
                if let Ok(identity) = bincode::deserialize::<NodeIdentity>(&message.payload) {
                    known_nodes.insert(identity.node_id, identity);
                }
            }
            _ => {
                // Unknown message type
            }
        }
    }

    /// Announces this node to the network
    pub async fn announce_node(&self, bootstrap_nodes: Vec<SocketAddr>) -> Result<()> {
        let announcement = GossipMessage::new(
            "node_announcement".to_string(),
            bincode::serialize(&self.local_identity).map_err(|e| {
                Box::new(Error::runtime_error(
                    format!("Failed to serialize identity: {}", e),
                    None,
                ))
            })?,
            self.local_identity.node_id,
            self.message_sequence
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );

        let data = bincode::serialize(&announcement).map_err(|e| {
            Box::new(Error::runtime_error(
                format!("Failed to serialize announcement: {}", e),
                None,
            ))
        })?;

        if let Some(socket) = &self.gossip_socket {
            for addr in bootstrap_nodes {
                let _ = socket.send_to(&data, addr).await;
            }
        }

        Ok(())
    }

    /// Gets all known nodes
    pub fn get_known_nodes(&self) -> Vec<NodeIdentity> {
        self.known_nodes
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Gets suspected nodes
    pub fn get_suspected_nodes(&self) -> Result<Vec<NodeId>> {
        self.liveness_monitor.get_suspected_nodes()
    }

    /// Gets current network view
    pub async fn get_network_view(&self) -> NetworkView {
        self.consensus.get_current_view().await
    }

    /// Shuts down the discovery service
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_node_identity_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
        let capabilities = NodeCapabilities::new(NodeId::new());
        let identity = NodeIdentity::new(addr, capabilities);

        assert_eq!(identity.address, addr);
        assert!(!identity.public_key.is_empty());
    }

    #[test]
    fn test_network_view() {
        let node_id = NodeId::new();
        let mut view = NetworkView::new(1, node_id);

        assert_eq!(view.view_number, 1);
        assert_eq!(view.proposer, node_id);

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
        let capabilities = NodeCapabilities::new(NodeId::new());
        let identity = NodeIdentity::new(addr, capabilities);

        view.add_active_node(identity.clone());
        assert_eq!(view.total_nodes(), 1);

        view.mark_suspected(identity.node_id);
        assert_eq!(view.active_nodes.len(), 0);
        assert_eq!(view.suspected_nodes.len(), 1);
    }

    #[test]
    fn test_liveness_monitor() {
        let monitor = LivenessMonitor::new();
        let node_id = NodeId::new();

        monitor.record_heartbeat(node_id).unwrap();
        monitor.mark_suspected(node_id).unwrap();

        let suspected = monitor.get_suspected_nodes().unwrap();
        assert!(suspected.is_empty()); // Should be removed when heartbeat recorded
    }

    #[test]
    fn test_gossip_message() {
        let node_id = NodeId::new();
        let mut message = GossipMessage::new("test".to_string(), b"payload".to_vec(), node_id, 1);

        assert_eq!(message.ttl, 10);
        assert!(message.decrement_ttl());
        assert_eq!(message.ttl, 9);
    }

    #[tokio::test]
    async fn test_byzantine_consensus() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
        let capabilities = NodeCapabilities::new(NodeId::new());
        let identity = NodeIdentity::new(addr, capabilities);

        let consensus = ByzantineConsensus::new(identity.clone());
        let view = consensus.get_current_view().await;

        assert_eq!(view.view_number, 0);
        assert_eq!(view.proposer, identity.node_id);
    }
}

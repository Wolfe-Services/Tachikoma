//! Connection tracking.

use dashmap::DashMap;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use uuid::Uuid;

/// Connection information.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    /// Connection ID.
    pub id: Uuid,
    /// Remote address.
    pub remote_addr: SocketAddr,
    /// Connection type.
    pub connection_type: ConnectionType,
    /// Connection timestamp.
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp.
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Bytes sent.
    pub bytes_sent: u64,
    /// Bytes received.
    pub bytes_received: u64,
}

/// Connection type.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Http,
    WebSocket,
}

/// Connection statistics.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ConnectionStats {
    /// Total active connections.
    pub active_connections: u64,
    /// HTTP connections.
    pub http_connections: u64,
    /// WebSocket connections.
    pub websocket_connections: u64,
    /// Total connections (lifetime).
    pub total_connections: u64,
    /// Total bytes sent.
    pub total_bytes_sent: u64,
    /// Total bytes received.
    pub total_bytes_received: u64,
}

/// Connection tracker for managing active connections.
pub struct ConnectionTracker {
    connections: DashMap<Uuid, ConnectionInfo>,
    total_connections: AtomicU64,
    total_bytes_sent: AtomicU64,
    total_bytes_received: AtomicU64,
    stats_sender: watch::Sender<ConnectionStats>,
    stats_receiver: watch::Receiver<ConnectionStats>,
}

impl ConnectionTracker {
    pub fn new() -> Self {
        let (stats_sender, stats_receiver) = watch::channel(ConnectionStats::default());
        Self {
            connections: DashMap::new(),
            total_connections: AtomicU64::new(0),
            total_bytes_sent: AtomicU64::new(0),
            total_bytes_received: AtomicU64::new(0),
            stats_sender,
            stats_receiver,
        }
    }

    /// Register a new connection.
    pub fn connect(&self, remote_addr: SocketAddr, connection_type: ConnectionType) -> Uuid {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let info = ConnectionInfo {
            id,
            remote_addr,
            connection_type,
            connected_at: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
        };

        self.connections.insert(id, info);
        self.total_connections.fetch_add(1, Ordering::SeqCst);
        self.update_stats();

        id
    }

    /// Update connection activity.
    pub fn activity(&self, id: Uuid, bytes_sent: u64, bytes_received: u64) {
        if let Some(mut conn) = self.connections.get_mut(&id) {
            conn.last_activity = chrono::Utc::now();
            conn.bytes_sent += bytes_sent;
            conn.bytes_received += bytes_received;
        }

        self.total_bytes_sent.fetch_add(bytes_sent, Ordering::SeqCst);
        self.total_bytes_received.fetch_add(bytes_received, Ordering::SeqCst);
        self.update_stats();
    }

    /// Disconnect a connection.
    pub fn disconnect(&self, id: Uuid) {
        self.connections.remove(&id);
        self.update_stats();
    }

    /// Get connection information.
    pub fn get_connection(&self, id: Uuid) -> Option<ConnectionInfo> {
        self.connections.get(&id).map(|conn| conn.clone())
    }

    /// List all active connections.
    pub fn list_connections(&self) -> Vec<ConnectionInfo> {
        self.connections.iter().map(|entry| entry.clone()).collect()
    }

    /// Get current statistics.
    pub fn stats(&self) -> ConnectionStats {
        self.stats_receiver.borrow().clone()
    }

    /// Subscribe to statistics updates.
    pub fn subscribe(&self) -> watch::Receiver<ConnectionStats> {
        self.stats_receiver.clone()
    }

    fn update_stats(&self) {
        let mut http_count = 0;
        let mut websocket_count = 0;

        for conn in self.connections.iter() {
            match conn.connection_type {
                ConnectionType::Http => http_count += 1,
                ConnectionType::WebSocket => websocket_count += 1,
            }
        }

        let stats = ConnectionStats {
            active_connections: self.connections.len() as u64,
            http_connections: http_count,
            websocket_connections: websocket_count,
            total_connections: self.total_connections.load(Ordering::SeqCst),
            total_bytes_sent: self.total_bytes_sent.load(Ordering::SeqCst),
            total_bytes_received: self.total_bytes_received.load(Ordering::SeqCst),
        };

        let _ = self.stats_sender.send(stats);
    }
}

impl Default for ConnectionTracker {
    fn default() -> Self {
        Self::new()
    }
}
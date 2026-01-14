# Spec 530: WireGuard Tunnel Management

## Overview
WireGuard VPN tunnel management for secure, encrypted communication between distributed Tachikoma agents across network boundaries.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### WireGuard Interface Types
```go
// WireGuardInterface represents a WG interface
type WireGuardInterface struct {
    Name       string            `json:"name"`
    PrivateKey wgtypes.Key       `json:"privateKey"`
    PublicKey  wgtypes.Key       `json:"publicKey"`
    ListenPort int               `json:"listenPort"`
    FwMark     int               `json:"fwMark,omitempty"`
    Peers      []WireGuardPeer   `json:"peers"`
    Address    net.IPNet         `json:"address"`
    MTU        int               `json:"mtu"`
}

// WireGuardPeer represents a remote peer
type WireGuardPeer struct {
    PublicKey           wgtypes.Key   `json:"publicKey"`
    PresharedKey        wgtypes.Key   `json:"presharedKey,omitempty"`
    Endpoint            *net.UDPAddr  `json:"endpoint,omitempty"`
    AllowedIPs          []net.IPNet   `json:"allowedIps"`
    PersistentKeepalive time.Duration `json:"persistentKeepalive"`
    LastHandshake       time.Time     `json:"lastHandshake"`
}
```

### Tunnel Manager Interface
```go
type TunnelManager interface {
    // Interface management
    CreateInterface(ctx context.Context, config InterfaceConfig) error
    DeleteInterface(ctx context.Context, name string) error
    GetInterface(ctx context.Context, name string) (*WireGuardInterface, error)

    // Peer management
    AddPeer(ctx context.Context, iface string, peer WireGuardPeer) error
    RemovePeer(ctx context.Context, iface string, publicKey wgtypes.Key) error
    UpdatePeer(ctx context.Context, iface string, peer WireGuardPeer) error

    // Status
    GetStats(ctx context.Context, iface string) (*InterfaceStats, error)
}
```

### Key Management
```go
type KeyManager interface {
    // Generate new keypair
    GenerateKeyPair() (wgtypes.Key, wgtypes.Key, error)

    // Load from secure storage
    LoadPrivateKey(id string) (wgtypes.Key, error)

    // Store securely
    StorePrivateKey(id string, key wgtypes.Key) error

    // Key rotation
    RotateKeys(ctx context.Context, iface string) error
}
```

### Automatic Peer Discovery
- Agent registration to coordinator
- Public key exchange
- Endpoint discovery
- NAT traversal
- Peer health monitoring

### Network Topology
```go
type MeshTopology struct {
    Nodes       []MeshNode        `json:"nodes"`
    Connections []MeshConnection  `json:"connections"`
    Coordinator string            `json:"coordinator"`
}

type MeshNode struct {
    ID        string    `json:"id"`
    PublicKey string    `json:"publicKey"`
    Endpoint  string    `json:"endpoint"`
    InternalIP string   `json:"internalIp"`
    Region    string    `json:"region"`
}
```

### NAT Traversal
- STUN server integration
- UDP hole punching
- Endpoint roaming detection
- Persistent keepalive tuning
- Fallback to relay (DERP)

### Security Features
- Perfect forward secrecy
- Preshared key support
- Key rotation scheduling
- Firewall integration
- Connection rate limiting

### Monitoring
- Handshake success rate
- Bandwidth per peer
- Latency measurements
- Connection drops
- Key expiration alerts

## Dependencies
- Spec 528: SPIFFE Identity
- Spec 531: DERP Relay

## Verification
- [ ] Tunnels establish correctly
- [ ] Peer discovery works
- [ ] NAT traversal functional
- [ ] Key rotation seamless
- [ ] Metrics collected

# Spec 531: DERP Relay Server

## Overview
DERP (Designated Encrypted Relay for Packets) server implementation for relaying encrypted WireGuard traffic when direct peer-to-peer connections fail.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### DERP Server Types
```go
// DERPServer represents a relay server
type DERPServer struct {
    ID          string    `json:"id"`
    Hostname    string    `json:"hostname"`
    IPv4        string    `json:"ipv4"`
    IPv6        string    `json:"ipv6,omitempty"`
    STUNPort    int       `json:"stunPort"`
    DERPPort    int       `json:"derpPort"`
    Region      string    `json:"region"`
    RegionCode  int       `json:"regionCode"`
    CanSTUN     bool      `json:"canStun"`
    CanRelay    bool      `json:"canRelay"`
}

// DERPMap contains all relay servers
type DERPMap struct {
    Regions map[int]*DERPRegion `json:"regions"`
}

type DERPRegion struct {
    RegionID   int           `json:"regionId"`
    RegionCode string        `json:"regionCode"`
    RegionName string        `json:"regionName"`
    Nodes      []*DERPServer `json:"nodes"`
}
```

### Relay Server Implementation
```go
type RelayServer interface {
    // Start server
    Start(ctx context.Context, config ServerConfig) error

    // Stop server
    Stop(ctx context.Context) error

    // Client connections
    AcceptClient(ctx context.Context) (*ClientConn, error)

    // Relay packet
    RelayPacket(ctx context.Context, from, to string, data []byte) error

    // Health check
    HealthCheck(ctx context.Context) error
}
```

### Client Connection
```go
type DERPClient interface {
    // Connect to relay
    Connect(ctx context.Context, server *DERPServer) error

    // Disconnect
    Disconnect() error

    // Send packet via relay
    Send(ctx context.Context, dstKey string, data []byte) error

    // Receive packets
    Receive(ctx context.Context) (*ReceivedPacket, error)

    // Get connected peers via this relay
    ConnectedPeers() []string
}
```

### STUN Server
- UDP STUN endpoint
- RFC 5389 compliance
- External address detection
- Binding request handling
- Response caching

### Relay Protocol
- WebSocket transport
- TLS encryption
- Frame-based messaging
- Keep-alive mechanism
- Reconnection handling

### Server Selection
```go
type ServerSelector interface {
    // Select best server for region
    SelectServer(region string, exclude []string) (*DERPServer, error)

    // Latency-based selection
    SelectByLatency(servers []*DERPServer) (*DERPServer, error)

    // Health-aware selection
    SelectHealthy(servers []*DERPServer) (*DERPServer, error)
}
```

### Deployment Configuration
```yaml
derp:
  servers:
    - region: us-east
      hostname: derp-use.tachikoma.dev
      stunPort: 3478
      derpPort: 443
    - region: eu-west
      hostname: derp-euw.tachikoma.dev
      stunPort: 3478
      derpPort: 443
```

### Monitoring
- Connected client count
- Packets relayed per second
- Bandwidth utilization
- Latency measurements
- Error rates

### Security
- TLS certificate management
- Client authentication
- Rate limiting per client
- DDoS protection
- Abuse detection

## Dependencies
- Spec 530: WireGuard Tunnel

## Verification
- [ ] STUN works correctly
- [ ] Relay establishes
- [ ] Failover functions
- [ ] Performance adequate
- [ ] Security measures active

# Spec 542: Multi-Agent Communication

## Overview
Communication protocol and message passing system for coordination between multiple Tachikoma agents, enabling distributed task execution and state synchronization.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Message Types
```go
// Message represents inter-agent communication
type Message struct {
    ID          string          `json:"id"`
    Type        MessageType     `json:"type"`
    From        string          `json:"from"`
    To          string          `json:"to"` // agent ID or broadcast
    Topic       string          `json:"topic,omitempty"`
    Payload     json.RawMessage `json:"payload"`
    Timestamp   time.Time       `json:"timestamp"`
    TTL         time.Duration   `json:"ttl,omitempty"`
    ReplyTo     string          `json:"replyTo,omitempty"`
    CorrelationID string        `json:"correlationId,omitempty"`
}

type MessageType string

const (
    MessageTypeRequest   MessageType = "request"
    MessageTypeResponse  MessageType = "response"
    MessageTypeEvent     MessageType = "event"
    MessageTypeBroadcast MessageType = "broadcast"
    MessageTypeHeartbeat MessageType = "heartbeat"
)
```

### Communication Interface
```go
type AgentCommunicator interface {
    // Send message to specific agent
    Send(ctx context.Context, msg *Message) error

    // Request-response pattern
    Request(ctx context.Context, msg *Message) (*Message, error)

    // Broadcast to all agents
    Broadcast(ctx context.Context, msg *Message) error

    // Subscribe to topic
    Subscribe(ctx context.Context, topic string) (<-chan *Message, error)

    // Unsubscribe from topic
    Unsubscribe(ctx context.Context, topic string) error

    // Receive messages
    Receive(ctx context.Context) (<-chan *Message, error)
}
```

### Transport Layer
```go
type Transport interface {
    // Connect to peer
    Connect(ctx context.Context, addr string) error

    // Disconnect from peer
    Disconnect(ctx context.Context, addr string) error

    // Send raw bytes
    Send(ctx context.Context, peer string, data []byte) error

    // Receive raw bytes
    Receive(ctx context.Context) (string, []byte, error)
}
```

### Transport Implementations
- gRPC streaming transport
- WebSocket transport
- NATS JetStream transport
- WireGuard mesh transport

### Message Routing
```go
type MessageRouter interface {
    // Register route handler
    Handle(topic string, handler MessageHandler) error

    // Route message
    Route(ctx context.Context, msg *Message) error

    // Get routing table
    Routes() map[string][]string
}

type MessageHandler func(ctx context.Context, msg *Message) (*Message, error)
```

### Pub/Sub System
```go
type PubSub interface {
    // Publish to topic
    Publish(ctx context.Context, topic string, payload interface{}) error

    // Subscribe to topic
    Subscribe(ctx context.Context, topic string) (<-chan *Message, error)

    // Pattern subscribe (wildcards)
    PSubscribe(ctx context.Context, pattern string) (<-chan *Message, error)

    // Unsubscribe
    Unsubscribe(ctx context.Context, topic string) error
}
```

### Message Serialization
```go
type Serializer interface {
    Serialize(msg *Message) ([]byte, error)
    Deserialize(data []byte) (*Message, error)
}

// Implementations
type JSONSerializer struct{}
type ProtobufSerializer struct{}
type MsgpackSerializer struct{}
```

### Delivery Guarantees
- At-most-once (fire and forget)
- At-least-once (with acknowledgment)
- Exactly-once (with deduplication)

### Message Queue
```go
type MessageQueue interface {
    // Enqueue message
    Enqueue(ctx context.Context, msg *Message) error

    // Dequeue message
    Dequeue(ctx context.Context) (*Message, error)

    // Acknowledge processing
    Ack(ctx context.Context, msgID string) error

    // Negative acknowledge (requeue)
    Nack(ctx context.Context, msgID string) error
}
```

### Security
- End-to-end encryption
- Message signing (SPIFFE)
- Replay attack prevention
- Rate limiting per agent

## Dependencies
- Spec 528: SPIFFE Identity
- Spec 530: WireGuard Tunnel

## Verification
- [ ] Messages delivered correctly
- [ ] Pub/sub works
- [ ] Request-response pattern works
- [ ] Encryption functional
- [ ] Transport interchangeable

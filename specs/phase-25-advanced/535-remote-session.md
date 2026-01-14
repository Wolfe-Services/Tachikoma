# Spec 535: Remote Session Management

## Overview
Interactive remote session management for Tachikoma agents, supporting PTY allocation, session multiplexing, and terminal emulation.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Session Types
```go
// Session represents a remote terminal session
type Session struct {
    ID            string        `json:"id"`
    AgentID       string        `json:"agentId"`
    UserID        string        `json:"userId"`
    State         SessionState  `json:"state"`
    PTY           *PTYConfig    `json:"pty,omitempty"`
    CreatedAt     time.Time     `json:"createdAt"`
    LastActivity  time.Time     `json:"lastActivity"`
    IdleTimeout   time.Duration `json:"idleTimeout"`
    MaxDuration   time.Duration `json:"maxDuration"`
}

type SessionState string

const (
    SessionStatePending   SessionState = "pending"
    SessionStateActive    SessionState = "active"
    SessionStateSuspended SessionState = "suspended"
    SessionStateClosed    SessionState = "closed"
)
```

### PTY Configuration
```go
type PTYConfig struct {
    Term    string `json:"term"`  // xterm-256color
    Rows    uint16 `json:"rows"`
    Cols    uint16 `json:"cols"`
    XPixel  uint16 `json:"xPixel"`
    YPixel  uint16 `json:"yPixel"`
}
```

### Session Manager Interface
```go
type SessionManager interface {
    // Session lifecycle
    Create(ctx context.Context, req *SessionRequest) (*Session, error)
    Attach(ctx context.Context, sessionID string) (*SessionConn, error)
    Detach(ctx context.Context, sessionID string) error
    Close(ctx context.Context, sessionID string) error

    // Session operations
    Resize(ctx context.Context, sessionID string, pty PTYConfig) error
    SendSignal(ctx context.Context, sessionID string, sig os.Signal) error

    // Session listing
    List(ctx context.Context, filter SessionFilter) ([]*Session, error)
    Get(ctx context.Context, sessionID string) (*Session, error)
}
```

### Session Connection
```go
type SessionConn interface {
    // I/O
    Read(p []byte) (n int, err error)
    Write(p []byte) (n int, err error)

    // PTY operations
    Resize(rows, cols uint16) error

    // Control
    Close() error
    Wait() error
}
```

### Session Multiplexing
```go
type SessionMultiplexer interface {
    // Create new window
    NewWindow(sessionID string) (*Window, error)

    // Switch active window
    SwitchWindow(sessionID string, windowID int) error

    // List windows
    ListWindows(sessionID string) ([]*Window, error)

    // Close window
    CloseWindow(sessionID string, windowID int) error
}

type Window struct {
    ID        int       `json:"id"`
    Name      string    `json:"name"`
    Active    bool      `json:"active"`
    CreatedAt time.Time `json:"createdAt"`
}
```

### Session Recording
```go
type SessionRecorder interface {
    // Start recording
    Start(sessionID string) error

    // Stop recording
    Stop(sessionID string) error

    // Playback
    Playback(recordingID string) (*Recording, error)
}

type Recording struct {
    ID         string        `json:"id"`
    SessionID  string        `json:"sessionId"`
    StartTime  time.Time     `json:"startTime"`
    Duration   time.Duration `json:"duration"`
    Size       int64         `json:"size"`
    Format     string        `json:"format"` // asciicast
}
```

### Session Sharing
- Read-only observer mode
- Collaborative editing mode
- Session handoff between users
- Access control per session

### Security
- Session token rotation
- Idle timeout enforcement
- Maximum session duration
- IP-based restrictions
- Session audit logging

### WebSocket Transport
```go
type WebSocketSession struct {
    Conn        *websocket.Conn
    Session     *Session
    Heartbeat   time.Duration
    Compression bool
}
```

## Dependencies
- Spec 534: Remote Execution API
- Spec 528: SPIFFE Identity

## Verification
- [ ] PTY allocation works
- [ ] Resize signals propagate
- [ ] Multiplexing functional
- [ ] Recording works
- [ ] Security enforced

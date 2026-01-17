# 487 - Load Testing Setup

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 487
**Status:** Planned
**Dependencies:** 471-test-harness, 316-server-crate
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement load testing infrastructure using k6 to stress test the Tachikoma server API endpoints and verify system behavior under high concurrency.

---

## Acceptance Criteria

- [x] k6 scripts for critical API endpoints
- [x] Configurable load profiles (smoke, load, stress, soak)
- [x] Response time and error rate thresholds
- [x] CI integration for regression detection
- [x] Grafana dashboards for result visualization
- [x] Documentation for running load tests

---

## Implementation Details

### 1. k6 Configuration

Create `load-tests/k6.config.js`:

```javascript
export const options = {
  // Thresholds
  thresholds: {
    // 95% of requests should complete within 500ms
    http_req_duration: ['p(95)<500'],
    // Error rate should be less than 1%
    http_req_failed: ['rate<0.01'],
    // 99% of requests should complete within 1500ms
    'http_req_duration{expected_response:true}': ['p(99)<1500'],
  },

  // Tags
  tags: {
    project: 'tachikoma',
  },
};

// Load test profiles
export const profiles = {
  // Quick smoke test
  smoke: {
    vus: 1,
    duration: '30s',
  },

  // Standard load test
  load: {
    stages: [
      { duration: '2m', target: 10 },  // Ramp up
      { duration: '5m', target: 10 },  // Hold
      { duration: '2m', target: 0 },   // Ramp down
    ],
  },

  // Stress test
  stress: {
    stages: [
      { duration: '2m', target: 10 },
      { duration: '5m', target: 50 },
      { duration: '2m', target: 100 },
      { duration: '5m', target: 100 },
      { duration: '5m', target: 0 },
    ],
  },

  // Soak test (long duration)
  soak: {
    stages: [
      { duration: '5m', target: 10 },
      { duration: '60m', target: 10 },
      { duration: '5m', target: 0 },
    ],
  },

  // Spike test
  spike: {
    stages: [
      { duration: '1m', target: 5 },
      { duration: '10s', target: 100 },  // Spike!
      { duration: '3m', target: 100 },
      { duration: '10s', target: 5 },
      { duration: '3m', target: 5 },
      { duration: '1m', target: 0 },
    ],
  },
};
```

### 2. API Load Test Scripts

Create `load-tests/tests/api-health.js`:

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const healthCheckDuration = new Trend('health_check_duration');

export const options = {
  thresholds: {
    errors: ['rate<0.01'],
    health_check_duration: ['p(95)<100'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export default function () {
  const response = http.get(`${BASE_URL}/api/health`);

  healthCheckDuration.add(response.timings.duration);

  const success = check(response, {
    'status is 200': (r) => r.status === 200,
    'response has status field': (r) => JSON.parse(r.body).status !== undefined,
  });

  errorRate.add(!success);

  sleep(0.5);
}
```

Create `load-tests/tests/api-missions.js`:

```javascript
import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const missionCreateDuration = new Trend('mission_create_duration');
const missionListDuration = new Trend('mission_list_duration');
const missionsCreated = new Counter('missions_created');

export const options = {
  thresholds: {
    errors: ['rate<0.05'],
    mission_create_duration: ['p(95)<1000'],
    mission_list_duration: ['p(95)<500'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const API_KEY = __ENV.API_KEY || 'test-api-key';

const headers = {
  'Content-Type': 'application/json',
  'Authorization': `Bearer ${API_KEY}`,
};

export default function () {
  group('Mission CRUD', () => {
    // Create mission
    const createPayload = JSON.stringify({
      name: `Load Test Mission ${Date.now()}`,
      prompt: 'This is a load test prompt',
      backend: 'mock',
    });

    const createResponse = http.post(
      `${BASE_URL}/api/missions`,
      createPayload,
      { headers }
    );

    missionCreateDuration.add(createResponse.timings.duration);

    const createSuccess = check(createResponse, {
      'create status is 201': (r) => r.status === 201,
      'create returns id': (r) => JSON.parse(r.body).id !== undefined,
    });

    if (createSuccess) {
      missionsCreated.add(1);
    }
    errorRate.add(!createSuccess);

    // List missions
    const listResponse = http.get(`${BASE_URL}/api/missions`, { headers });

    missionListDuration.add(listResponse.timings.duration);

    const listSuccess = check(listResponse, {
      'list status is 200': (r) => r.status === 200,
      'list returns array': (r) => Array.isArray(JSON.parse(r.body)),
    });

    errorRate.add(!listSuccess);
  });

  sleep(1);
}
```

Create `load-tests/tests/api-specs.js`:

```javascript
import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const specGetDuration = new Trend('spec_get_duration');
const specSearchDuration = new Trend('spec_search_duration');

export const options = {
  thresholds: {
    errors: ['rate<0.05'],
    spec_get_duration: ['p(95)<300'],
    spec_search_duration: ['p(95)<500'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const API_KEY = __ENV.API_KEY || 'test-api-key';

const headers = {
  'Authorization': `Bearer ${API_KEY}`,
};

// Pre-defined spec IDs for testing
const specIds = ['001', '002', '003', '010', '020', '050', '100'];

export default function () {
  group('Spec Operations', () => {
    // Get random spec
    const specId = specIds[Math.floor(Math.random() * specIds.length)];
    const getResponse = http.get(`${BASE_URL}/api/specs/${specId}`, { headers });

    specGetDuration.add(getResponse.timings.duration);

    const getSuccess = check(getResponse, {
      'get status is 200 or 404': (r) => r.status === 200 || r.status === 404,
    });
    errorRate.add(!getSuccess);

    // Search specs
    const searchTerms = ['config', 'test', 'loop', 'backend', 'ui'];
    const searchTerm = searchTerms[Math.floor(Math.random() * searchTerms.length)];

    const searchResponse = http.get(
      `${BASE_URL}/api/specs/search?q=${searchTerm}`,
      { headers }
    );

    specSearchDuration.add(searchResponse.timings.duration);

    const searchSuccess = check(searchResponse, {
      'search status is 200': (r) => r.status === 200,
      'search returns array': (r) => Array.isArray(JSON.parse(r.body)),
    });
    errorRate.add(!searchSuccess);
  });

  sleep(0.5);
}
```

### 3. WebSocket Load Test

Create `load-tests/tests/websocket.js`:

```javascript
import ws from 'k6/ws';
import { check } from 'k6';
import { Rate, Counter } from 'k6/metrics';

const errorRate = new Rate('ws_errors');
const messagesReceived = new Counter('ws_messages_received');
const messagesSent = new Counter('ws_messages_sent');

export const options = {
  thresholds: {
    ws_errors: ['rate<0.01'],
  },
};

const WS_URL = __ENV.WS_URL || 'ws://localhost:3000/ws';
const API_KEY = __ENV.API_KEY || 'test-api-key';

export default function () {
  const response = ws.connect(
    `${WS_URL}?token=${API_KEY}`,
    {},
    function (socket) {
      socket.on('open', () => {
        // Subscribe to mission updates
        socket.send(JSON.stringify({
          type: 'subscribe',
          channel: 'missions',
        }));
        messagesSent.add(1);
      });

      socket.on('message', (msg) => {
        messagesReceived.add(1);

        const data = JSON.parse(msg);
        check(data, {
          'message has type': (d) => d.type !== undefined,
        });
      });

      socket.on('error', (e) => {
        errorRate.add(1);
        console.error('WebSocket error:', e);
      });

      // Keep connection open for 30 seconds
      socket.setTimeout(() => {
        socket.close();
      }, 30000);
    }
  );

  check(response, {
    'WebSocket connection successful': (r) => r && r.status === 101,
  });
}
```

### 4. Load Test Runner Script

Create `scripts/load-test.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

PROFILE="${1:-smoke}"
TEST="${2:-all}"
BASE_URL="${BASE_URL:-http://localhost:3000}"

echo "Running load test: profile=$PROFILE, test=$TEST"

# Start server if not running
if ! curl -s "$BASE_URL/api/health" > /dev/null 2>&1; then
    echo "Starting server..."
    cargo run --release -p tachikoma-server &
    SERVER_PID=$!
    sleep 5
fi

# Run tests
case $TEST in
    health)
        k6 run --env BASE_URL="$BASE_URL" \
            -e PROFILE="$PROFILE" \
            load-tests/tests/api-health.js
        ;;
    missions)
        k6 run --env BASE_URL="$BASE_URL" \
            -e PROFILE="$PROFILE" \
            load-tests/tests/api-missions.js
        ;;
    specs)
        k6 run --env BASE_URL="$BASE_URL" \
            -e PROFILE="$PROFILE" \
            load-tests/tests/api-specs.js
        ;;
    websocket)
        k6 run --env WS_URL="ws://localhost:3000/ws" \
            -e PROFILE="$PROFILE" \
            load-tests/tests/websocket.js
        ;;
    all)
        for test in health missions specs; do
            echo "Running $test tests..."
            k6 run --env BASE_URL="$BASE_URL" \
                -e PROFILE="$PROFILE" \
                "load-tests/tests/api-$test.js"
        done
        ;;
    *)
        echo "Unknown test: $TEST"
        exit 1
        ;;
esac

# Cleanup
if [ -n "${SERVER_PID:-}" ]; then
    kill $SERVER_PID 2>/dev/null || true
fi

echo "Load tests complete!"
```

### 5. CI Integration

Add to `.github/workflows/load-tests.yml`:

```yaml
name: Load Tests

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
  workflow_dispatch:

jobs:
  load-test:
    runs-on: ubuntu-latest
    services:
      app:
        image: tachikoma:latest
        ports:
          - 3000:3000

    steps:
      - uses: actions/checkout@v4

      - name: Install k6
        run: |
          sudo gpg -k
          sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6

      - name: Run load tests
        run: ./scripts/load-test.sh load all

      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: load-test-results
          path: load-tests/results/
```

---

## Testing Requirements

1. k6 scripts execute without errors
2. Thresholds accurately detect performance issues
3. CI integration runs reliably
4. Results are properly recorded
5. Documentation covers all test profiles

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [316-server-crate.md](../phase-15-server/316-server-crate.md)
- Next: [488-test-ci.md](488-test-ci.md)
- Related: [486-benchmarks.md](486-benchmarks.md)

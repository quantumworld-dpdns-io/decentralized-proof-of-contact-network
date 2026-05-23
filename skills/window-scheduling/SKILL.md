---
name: Window Scheduling
description: Schedule and manage orbital windows for proof-of-contact creation
trigger: When the user asks about scheduling windows or managing orbital windows
---

# Window Scheduling Skill

Orbital windows define time periods during which proofs-of-contact can be created. Windows come in three types:

- **Standard** — Fixed hourly or daily windows for routine proof generation
- **Extended** — Longer windows (configurable hours) for relaxed schedules
- **Emergency** — Short windows (minutes) for urgent proofs

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/windows` | List all windows |
| `GET /api/v1/windows/active` | List currently active windows |
| `POST /api/v1/windows` | Create a new window |
| `GET /api/v1/windows/{id}` | Get window details |

## Python SDK Usage

```python
from datetime import datetime, timedelta, timezone
from poi import PoiClient

client = PoiClient(base_url="http://localhost:3000")

# List active windows
windows = client.get_active_windows()
for w in windows:
    print(f"Window {w.id}: {w.window_type} ({w.start_time} - {w.end_time})")

# Create a custom window
from datetime import datetime, timedelta, timezone
window = client.create_window(
    start_time=datetime.now(timezone.utc).isoformat(),
    end_time=(datetime.now(timezone.utc) + timedelta(hours=4)).isoformat(),
    window_type="extended",
)
print(f"Created window {window.id}")
```

## Using the Scheduling Script

```bash
python skills/window-scheduling/scripts/schedule.py --api-url http://localhost:3000 create --type extended --duration 4
```

## Window Types Reference

| Type | Default Duration | Use Case |
|------|-----------------|----------|
| `standard` | 1 hour | Routine contact proofs |
| `extended` | Configurable (hours) | Flexible scheduling |
| `emergency` | 30 minutes | Urgent proof generation |

## Troubleshooting

- **No active windows**: Create a new window or check if existing windows have expired
- **Window creation fails**: Ensure start_time is before end_time and duration exceeds 0
- **Proof rejected with "Window expired"**: The window has closed; create a new proof in an active window

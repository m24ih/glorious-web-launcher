# glorious-service

Local HID service for Glorious gaming mice on Linux, with a small HTTP and
WebSocket API. It talks to the vendor HID feature-report interface directly
(`hidapi`) and does not depend on Electron, node-hid or Windows DLLs.

The device catalog is the repository's `data/glorious_devices.json` (extracted
from Glorious CORE's `SupportDevice.DB`), so every VID/PID pair in that catalog
is recognised: Model O / O- / O PRO / D / D- / D PRO / O 2 / D 2 / Series One
Pro wireless, Model O wired, Model I, Model I 2 Wireless, Model O 2 Wireless,
and the 4k/8kHz editions.

## Status

| Family | Usage page | Implementation |
|---|---|---|
| Sinowealth (Model O / D series) | `0xFFFF` | Firmware, battery, profile, DPI, polling, debounce, LOD, sleep, RGB |
| PixArt (Model I 2, Model O 2) | `0xFF00` / `0xFF01` | Recognised and listed; command set scaffolded in `src/hid/pixart.rs` |

Live verification on this machine (Model O Wireless dongle `0x258A:0x2022`,
interface 2, `/dev/hidraw2`):

```text
fw=0.3.8.1 (dongle), 0.3.8.0 (mouse), battery=85%, profile=1, polling=1000 Hz,
debounce=12 ms, sleep=600 s, 4 DPI stages
```

## Build and run

```sh
cargo build
cargo test
cargo run -p glorious-service -- --probe     # one-shot JSON + summary
cargo run -p glorious-service                # HTTP + WS on 127.0.0.1:25820
```

Options:

| Flag | Env | Default |
|---|---|---|
| `--bind <ADDR>` | `GLORIOUS_BIND` | `127.0.0.1:25820` |
| `--devices-json <PATH>` | `GLORIOUS_DEVICES_JSON` | `data/glorious_devices.json` |
| `--delay-ms <MS>` | `GLORIOUS_HID_DELAY_MS` | `80` |
| `--probe` | — | read firmware/battery/settings once and exit |
| `--no-browser` | — | do not open the control panel on startup |
| `--no-desktop` | — | disable Linux tray and desktop notifications |
| `-v` | — | debug logging |

The database is resolved from the current directory, the workspace root, or
next to the executable. No configuration file is required.

### Linux desktop integration

The service registers a StatusNotifierItem tray icon and continues polling while
it runs, even without an open browser. Click the icon to open the control panel;
the menu shows device status and can stop the service. Desktop notifications
(via `notify-send`) report device connection changes, charging, and low battery
at 20%, 10%, and 5%. A compatible tray host, a session D-Bus, and `notify-send`
are needed for the corresponding desktop features; the HTTP service still runs
without them. `--no-desktop` disables both tray and notifications for headless use.
An asleep or out-of-range wireless mouse may still have a connected receiver;
the service cannot distinguish those states from receiver telemetry alone.

### Permissions

Install `packaging/99-glorious.rules` (see `packaging/README.md`) so the active
session can open `/dev/hidraw*`. Without it, opening the vendor interface fails
with a permission error and devices are reported with `status: "error"`.

## API

`GET /api/health`

```json
{"status":"ok","service":"glorious-service","version":"0.1.0","uptime_seconds":12,"ws_subscribers":1}
```

`GET /api/devices` — connected devices only:

```json
{
  "count": 1,
  "timestamp": 1793626756,
  "devices": [
    {
      "slug": "model-o-wireless",
      "name": "MODEL O WIRELESS",
      "id": "JiVthmhNA6BGVZnS",
      "routerID": "ModelOSeries",
      "protocol": "sinowealth",
      "supported": true,
      "connected": true,
      "status": "connected",
      "connection": {
        "type": "Dongle", "vid": "0x258A", "pid": "0x2022",
        "interface": 2, "usage_page": "0xFFFF", "usage": "0x0000",
        "path": "/dev/hidraw2"
      },
      "image": "assets/images/ModelO3.png",
      "battery": {"percentage": 85, "charging": false, "status": "ok"},
      "firmware": {"dongle": "0.3.8.1", "mouse": "0.3.8.0", "display": "0.3.8.1"},
      "last_seen": 1793626756
    }
  ]
}
```

`GET /api/device/{slug}` — same object plus `settings` (live reads):

```json
"settings": {
  "active_profile": 1,
  "dpi_stage_count": 4,
  "dpi_stages": [],
  "active_dpi_stage": 4,
  "polling_rate": 1000,
  "debounce_ms": 12,
  "lod": 1,
  "sleep_seconds": 600
}
```

If the slug is known but the product is not plugged in, the response is `200`
with `connected: false`, `status: "disconnected"` and no telemetry. Unknown
slugs return `404`.

`POST /api/device/{slug}/apply` — apply a partial settings patch:

```sh
curl -X POST http://127.0.0.1:25820/api/device/model-o-wireless/apply \
  -H 'content-type: application/json' \
  -d '{
        "profile": 1,
        "dpi_stages": [800, 1600, 3200, 6400],
        "active_dpi_stage": 2,
        "polling_rate": 1000,
        "debounce_ms": 4,
        "lod": 1,
        "sleep_seconds": 600,
        "rgb": {
          "effect": "breathing",
          "brightness": 70,
          "rate": 50,
          "color": [255, 0, 0]
        }
      }'
```

Response: `{ "slug": ..., "applied": ["profile", "dpi_stages", ...], "device": {...} }`.

Validation happens before any HID write (422/400-friendly errors):

* `profile` 1-3, `dpi_stages` 1-8 entries with 100-30000 DPI,
* `active_dpi_stage` 1-8 and within the configured stage count,
* `polling_rate` one of 125/250/500/1000/2000/4000/8000,
* `debounce_ms` 0-50, `lod` 1 or 2, `sleep_seconds` 0-65535,
* `rgb.effect` id 0-8 or name (`off`, `glorious`, `seamless_breathing`,
  `breathing`, `static`, `single_breathing`, `tail`, `rave`, `wave`),
  brightness 0-100, rate 1-100.

`WS /ws` — on connect the client receives a snapshot, then a telemetry event
every 3 seconds and an `applied` event after successful writes:

```json
{"type":"snapshot","ts":1793626756,"devices":[...]}
{"type":"telemetry","ts":1793626759,"devices":[...]}
{"type":"applied","ts":1793626766,"slug":"model-o-wireless","applied":["polling_rate"],"device":{...}}
{"type":"error","ts":...,"message":"..."}
```

A client may send `{"type":"refresh"}` to request an immediate snapshot. CORS is
permissive so a local web UI served from another port can connect.

## Protocol notes (Sinowealth, Model O / D)

65-byte feature report, report id `0x00`, sent via `HIDIOCSFEATURE` and read via
`HIDIOCGFEATURE`:

```text
request : 00 00 00 TT LL CC OO payload...
          TT target (00 dongle, 02 mouse)   LL payload length
          CC category (00 system, 01 sensor, 02 RGB, 03 buttons, 04 macro)
          OO opcode

response: 00 SS 00 TT LL CC OO payload...   (before report-id stripping)
          SS: A1 ok, A0 busy, A2 failure, A3 unsupported, A4 no RF link
```

The dongle needs roughly 50-100 ms between the write and the read; the service
uses 80 ms by default and retries busy/failure responses. On Linux the vendor
collection for the `0x258A:0x2022` dongle is interface 2, usage page `0xFFFF`,
usage `0x00` — the endpoint selector prefers the database's `hid.set` pair and
then falls back to interface 2 / `0xFFFF:0x00`.

Implemented opcodes:

| Area | Category | Opcode | Direction |
|---|---|---|---|
| Firmware | `0x00` | `0x81` | read (target 0 = dongle, 2 = mouse) |
| Battery (`charging`, `%`) | `0x00` | `0x83` | read |
| Active profile / set profile | `0x00` | `0x85` / `0x05` | read / write |
| Sleep timer | `0x00` | `0x87` / `0x07` | read / write |
| Debounce | `0x00` | `0x88` / `0x08` | read / write |
| Polling rate | `0x01` | `0x80` / `0x00` (`0x0A` for 2 Pro generation) | read / write |
| DPI values | `0x01` | `0x81` / `0x01` | read / write |
| Active DPI stage | `0x01` | `0x82` / `0x02` | read / write |
| Lift-off distance | `0x01` | `0x88` / `0x08` | read / write |
| RGB effect / brightness | `0x02` | `0x00` / `0x02` | write |

### DPI readback quirk

The tested Model O Wireless firmware (dongle `0.3.8.1`, mouse `0.3.8.0`) answers
the DPI query with the stage count (4) but zeroed resolution bytes. Glorious
CORE has the same workaround and keeps the local database values. This service
therefore returns `dpi_stage_count` and leaves `dpi_stages` empty rather than
inventing values; writes still work. The UI should keep the user's intended
stage values after an apply.

## Layout

```text
Cargo.toml                      workspace
crates/glorious-service/
  src/main.rs                   CLI, probe mode, Axum server startup
  src/api.rs                    routes, state, error mapping
  src/ws.rs                     WebSocket broadcast + 3 s telemetry loop
  src/manager.rs                DB ↔ HID detection, snapshots, apply
  src/devices.rs                glorious_devices.json model + VID/PID lookup
  src/types.rs                  API/DTO types and patch validation
  src/hid/mod.rs                endpoint enumeration and vendor interface choice
  src/hid/sinowealth.rs         Model O / D 65-byte protocol
  src/hid/pixart.rs             Model I 2 / O 2 scaffold
packaging/99-glorious.rules     udev access rule
```

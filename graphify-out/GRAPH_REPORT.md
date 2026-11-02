# Graph Report - Glorious-Web-Launcher  (2026-11-02)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 396 nodes · 965 edges · 11 communities (10 shown, 1 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `bb1567ad`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Community 0
- Community 1
- Community 2
- Community 3
- Community 4
- Community 5
- Community 6
- Community 7
- Community 8
- Community 9
- Community 10

## God Nodes (most connected - your core abstractions)
1. `HidError` - 52 edges
2. `SinowealthDevice` - 33 edges
3. `DeviceEntry` - 24 edges
4. `Response` - 23 edges
5. `DeviceManager` - 22 edges
6. `DeviceView` - 20 edges
7. `Endpoint` - 20 edges
8. `ensure_success()` - 16 edges
9. `AppState` - 15 edges
10. `DeviceDatabase` - 15 edges

## Surprising Connections (you probably didn't know these)
- `ApiError` --references--> `ManagerError`  [EXTRACTED]
  crates/glorious-service/src/api.rs → crates/glorious-service/src/manager.rs
- `DeviceList` --references--> `DeviceView`  [EXTRACTED]
  crates/glorious-service/src/api.rs → crates/glorious-service/src/types.rs
- `WsEvent` --references--> `DeviceView`  [EXTRACTED]
  crates/glorious-service/src/ws.rs → crates/glorious-service/src/types.rs
- `ManagerError` --references--> `HidError`  [EXTRACTED]
  crates/glorious-service/src/manager.rs → crates/glorious-service/src/hid/mod.rs
- `SinowealthDevice` --references--> `ConnectionKind`  [EXTRACTED]
  crates/glorious-service/src/hid/sinowealth.rs → crates/glorious-service/src/hid/mod.rs

## Import Cycles
- 2-file cycle: `crates/glorious-service/src/api.rs -> crates/glorious-service/src/ws.rs -> crates/glorious-service/src/api.rs`

## Communities (11 total, 1 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.06
Nodes (50): axum, Box, broadcast, corslayer, ApiError, apply_device(), AppState, DEFAULT_BIND (+42 more)

### Community 1 - "Community 1"
Cohesion: 0.05
Nodes (52): ACTIVE_DPI_GET, ACTIVE_DPI_SET, BATTERY, battery_report(), brightness_to_byte(), build_request(), builds_65_byte_request(), CATEGORY_BUTTONS (+44 more)

### Community 2 - "Community 2"
Cohesion: 0.14
Nodes (24): HidError, Error, From, DpiInfo, ensure_success(), hid_to_polling_rate(), parse_active_dpi_stage(), parse_battery() (+16 more)

### Community 3 - "Community 3"
Cohesion: 0.09
Nodes (30): AsRef, classifies_model_i_2_as_pixart_scaffold(), Connection, DbError, default_candidates(), DEFAULT_RELATIVE_PATH, DeviceDatabase, DeviceEntry (+22 more)

### Community 4 - "Community 4"
Cohesion: 0.11
Nodes (29): batteryinfo_as_battery, ApplyResult, base_view(), cache_key(), cache_key_is_stable(), database_has_model_o_wireless(), Detected, DeviceManager (+21 more)

### Community 5 - "Community 5"
Cohesion: 0.10
Nodes (21): BatteryInfo, ConnectionView, DeviceStatusView, DeviceView, EffectName, EffectSpec, FirmwareInfo, rejects_active_stage_beyond_stage_count() (+13 more)

### Community 6 - "Community 6"
Cohesion: 0.12
Nodes (22): choose_endpoint(), DEFAULT_DELAY, Endpoint, endpoints_for(), enumerate(), open_vendor_device(), REPORT_ID, REPORT_LEN (+14 more)

### Community 7 - "Community 7"
Cohesion: 0.11
Nodes (11): ConnectionKind, DeviceStatus, Self, PixArtCapabilities, PixArtDevice, HidApi, Result, Self (+3 more)

### Community 8 - "Community 8"
Cohesion: 0.18
Nodes (20): default_delay(), main(), Options, parse_delay(), print_help(), probe(), Arc, Duration (+12 more)

### Community 9 - "Community 9"
Cohesion: 0.18
Nodes (18): Any, argparse, datetime, Path, pathlib, re, build_device(), connections() (+10 more)

## Knowledge Gaps
- **33 isolated node(s):** `DEFAULT_BIND`, `ACTIVE_DPI_GET`, `ACTIVE_DPI_SET`, `BATTERY`, `CATEGORY_BUTTONS` (+28 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 104 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `HidError` connect `Community 2` to `Community 4`, `Community 6`, `Community 7`?**
  _High betweenness centrality (0.125) - this node is a cross-community bridge._
- **Why does `Response` connect `Community 2` to `Community 0`, `Community 1`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **Why does `SinowealthDevice` connect `Community 2` to `Community 1`, `Community 4`, `Community 6`, `Community 7`?**
  _High betweenness centrality (0.078) - this node is a cross-community bridge._
- **What connects `DEFAULT_BIND`, `ACTIVE_DPI_GET`, `ACTIVE_DPI_SET` to the rest of the system?**
  _33 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.061952074810052604 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05325814536340852 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.14395527603074773 - nodes in this community are weakly interconnected._
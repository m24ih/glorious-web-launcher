# Graph Report - Glorious-Web-Launcher  (2026-11-03)

## Corpus Check
- 34 files · ~1,307,865 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 2, .css 1, .rules 1)

## Summary
- 644 nodes · 1343 edges · 25 communities (22 shown, 3 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `413cf83f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- ws.rs
- sinowealth.rs
- HidError
- devices.rs
- manager.rs
- package.json
- Endpoint
- actions.ts
- main.rs
- extract_devices.py
- glorious-service
- glorious-service
- packaging
- types.ts
- compilerOptions
- api.ts
- profiles.rs
- apply_device
- api.rs
- ApiError
- EffectSpec
- types.rs
- AppState
- DeviceList

## God Nodes (most connected - your core abstractions)
1. `HidError` - 53 edges
2. `SinowealthDevice` - 34 edges
3. `DeviceManager` - 25 edges
4. `Response` - 24 edges
5. `DeviceEntry` - 24 edges
6. `ProfileStore` - 21 edges
7. `Endpoint` - 20 edges
8. `DeviceDatabase` - 16 edges
9. `ensure_success()` - 16 edges
10. `AppState` - 15 edges

## Surprising Connections (you probably didn't know these)
- `import_glorious_core_db_from_str()` --calls--> `effect_name_for()`  [INFERRED]
  crates/glorious-service/src/profiles.rs → crates/glorious-service/src/types.rs
- `ProfileConfig` --references--> `LightingView`  [EXTRACTED]
  crates/glorious-service/src/profiles.rs → crates/glorious-service/src/types.rs
- `DeviceManager` --references--> `ProfileStore`  [EXTRACTED]
  crates/glorious-service/src/manager.rs → crates/glorious-service/src/profiles.rs
- `ManagerError` --references--> `HidError`  [EXTRACTED]
  crates/glorious-service/src/manager.rs → crates/glorious-service/src/hid/mod.rs
- `static_handler()` --references--> `Response`  [EXTRACTED]
  crates/glorious-service/src/api.rs → crates/glorious-service/src/hid/sinowealth.rs

## Import Cycles
- 2-file cycle: `crates/glorious-service/src/api.rs -> crates/glorious-service/src/ws.rs -> crates/glorious-service/src/api.rs`

## Communities (25 total, 3 thin omitted)

### Community 0 - "ws.rs"
Cohesion: 0.14
Nodes (20): Box, broadcast, handle_socket(), DeviceView, Duration, IntoResponse, Result, State (+12 more)

### Community 1 - "sinowealth.rs"
Cohesion: 0.06
Nodes (50): ACTIVE_DPI_GET, ACTIVE_DPI_SET, BATTERY, battery_report(), build_request(), builds_65_byte_request(), builds_dpi_color_request(), CATEGORY_BUTTONS (+42 more)

### Community 2 - "HidError"
Cohesion: 0.11
Nodes (32): HidError, Error, From, brightness_to_byte(), DpiInfo, ensure_success(), hid_to_polling_rate(), parse_active_dpi_stage() (+24 more)

### Community 3 - "devices.rs"
Cohesion: 0.07
Nodes (34): AsRef, classifies_model_i_2_as_pixart_scaffold(), Connection, DbError, default_candidates(), DEFAULT_RELATIVE_PATH, DeviceEntry, DeviceImage (+26 more)

### Community 4 - "manager.rs"
Cohesion: 0.11
Nodes (31): batteryinfo_as_battery, DeviceDatabase, ApplyResult, cache_key(), cache_key_is_stable(), database_has_model_o_wireless(), device_view_includes_profile_settings(), DeviceManager (+23 more)

### Community 5 - "package.json"
Cohesion: 0.06
Nodes (34): dependencies, @fontsource-variable/inter, lucide-svelte, description, devDependencies, svelte, svelte-check, @sveltejs/vite-plugin-svelte (+26 more)

### Community 6 - "Endpoint"
Cohesion: 0.06
Nodes (34): choose_endpoint(), ConnectionKind, DEFAULT_DELAY, DeviceStatus, Endpoint, endpoints_for(), enumerate(), open_vendor_device() (+26 more)

### Community 7 - "actions.ts"
Cohesion: 0.09
Nodes (24): ALPHABET, clamp(), DEFAULT_MAPPINGS, DEFAULT_PIN_ACTIONS, DOCK_ACTIONS, DOCK_CATEGORIES, DPI_COLORS, draftFromDevice() (+16 more)

### Community 8 - "main.rs"
Cohesion: 0.14
Nodes (26): api, default_delay(), main(), open_browser(), Options, options_default_has_open_browser_enabled(), options_no_browser_flag_disables_browser(), options_open_flag_enables_browser() (+18 more)

### Community 9 - "extract_devices.py"
Cohesion: 0.09
Nodes (30): Any, argparse, datetime, build_render(), main(), Path, Build web-ready mouse renders from the extracted Glorious CORE assets. The…, Make the opaque white studio plate transparent, keeping the product. (+22 more)

### Community 11 - "glorious-service"
Cohesion: 0.15
Nodes (11): API, Build and run, DPI readback quirk, glorious-service, Layout, Permissions, Protocol notes (Sinowealth, Model O / D), Status (+3 more)

### Community 13 - "types.ts"
Cohesion: 0.11
Nodes (18): ActionCategory, ApplyResult, BatteryInfo, ConnectionView, DeviceList, DeviceStatus, DeviceView, DockAction (+10 more)

### Community 14 - "compilerOptions"
Cohesion: 0.13
Nodes (14): compilerOptions, allowJs, checkJs, isolatedModules, module, moduleDetection, moduleResolution, noEmit (+6 more)

### Community 16 - "api.ts"
Cohesion: 0.18
Nodes (10): API_BASE, ApiError, applySettings(), connectWs(), fetchDevice(), fetchDevices(), fetchHealth(), JSON_HEADERS (+2 more)

### Community 17 - "profiles.rs"
Cohesion: 0.10
Nodes (32): BTreeMap, DeviceProfileConfig, falls_back_to_defaults(), find_glorious_core_db(), import_glorious_core_db(), import_glorious_core_db_from_str(), imports_from_glorious_core_db_text(), ProfileConfig (+24 more)

### Community 18 - "apply_device"
Cohesion: 0.18
Nodes (17): ApiError, ApplyResult, apply_device(), device_details(), health(), list_devices(), Path, Result (+9 more)

### Community 19 - "api.rs"
Cohesion: 0.15
Nodes (15): axum, corslayer, DEFAULT_BIND, EmbeddedAssets, router(), Arc, static_known_asset_served_with_correct_mime(), static_root_serves_index_html() (+7 more)

### Community 20 - "ApiError"
Cohesion: 0.29
Nodes (6): ApiError, From, IntoResponse, Self, JoinError, StatusCode

### Community 21 - "EffectSpec"
Cohesion: 0.67
Nodes (3): EffectSpec, EffectId, EffectName

### Community 22 - "types.rs"
Cohesion: 0.09
Nodes (25): BatteryInfo, ConnectionView, DeviceStatusView, DeviceView, effect_name_for(), EffectName, EffectSpec, FirmwareInfo (+17 more)

### Community 23 - "AppState"
Cohesion: 0.40
Nodes (5): AppState, Instant, Sender, SharedManager, WsEvent

### Community 27 - "DeviceList"
Cohesion: 0.67
Nodes (3): DeviceList, DeviceView, Vec

## Knowledge Gaps
- **114 isolated node(s):** `ApplyResult`, `BatteryInfo`, `ConnectionView`, `DeviceList`, `DeviceStatus` (+109 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 226 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `HidError` connect `HidError` to `devices.rs`, `manager.rs`, `Endpoint`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Why does `Response` connect `HidError` to `sinowealth.rs`, `apply_device`, `api.rs`?**
  _High betweenness centrality (0.059) - this node is a cross-community bridge._
- **Why does `ProfileStore` connect `profiles.rs` to `manager.rs`?**
  _High betweenness centrality (0.049) - this node is a cross-community bridge._
- **What connects `ApplyResult`, `BatteryInfo`, `ConnectionView` to the rest of the system?**
  _114 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ws.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._
- **Should `sinowealth.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05656565656565657 - nodes in this community are weakly interconnected._
- **Should `HidError` be split into smaller, more focused modules?**
  _Cohesion score 0.11422527763088314 - nodes in this community are weakly interconnected._
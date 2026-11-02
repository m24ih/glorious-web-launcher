# Graph Report - Glorious-Web-Launcher  (2026-11-03)

## Corpus Check
- 34 files · ~1,310,078 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 2, .css 1, .rules 1)

## Summary
- 679 nodes · 1380 edges · 35 communities (22 shown, 13 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `343faadc`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- ws.rs
- sinowealth.rs
- Result
- devices.rs
- manager.rs
- package.json
- mod.rs
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
- BatteryInfo
- Duration
- FirmwareInfo
- DeviceList
- HidApi
- HidDevice
- Error
- Mutex
- Path
- PathBuf
- DeviceStatus

## God Nodes (most connected - your core abstractions)
1. `SinowealthDevice` - 35 edges
2. `DeviceManager` - 25 edges
3. `Response` - 24 edges
4. `DeviceEntry` - 23 edges
5. `ProfileStore` - 20 edges
6. `Endpoint` - 18 edges
7. `HidError` - 17 edges
8. `ensure_success()` - 16 edges
9. `DeviceDatabase` - 16 edges
10. `AppState` - 15 edges

## Surprising Connections (you probably didn't know these)
- `import_glorious_core_db_from_str()` --calls--> `effect_name_for()`  [INFERRED]
  crates/glorious-service/src/profiles.rs → crates/glorious-service/src/types.rs
- `static_handler()` --references--> `Response`  [EXTRACTED]
  crates/glorious-service/src/api.rs → crates/glorious-service/src/hid/sinowealth.rs
- `ProfileConfig` --references--> `LightingView`  [EXTRACTED]
  crates/glorious-service/src/profiles.rs → crates/glorious-service/src/types.rs
- `import_glorious_core_db_from_str()` --calls--> `normalize_hex_color()`  [INFERRED]
  crates/glorious-service/src/profiles.rs → crates/glorious-service/src/types.rs
- `ManagerError` --references--> `HidError`  [EXTRACTED]
  crates/glorious-service/src/manager.rs → crates/glorious-service/src/hid/mod.rs

## Import Cycles
- 2-file cycle: `crates/glorious-service/src/api.rs -> crates/glorious-service/src/ws.rs -> crates/glorious-service/src/api.rs`

## Communities (35 total, 13 thin omitted)

### Community 0 - "ws.rs"
Cohesion: 0.14
Nodes (20): Box, broadcast, handle_socket(), DeviceView, Duration, IntoResponse, Result, State (+12 more)

### Community 1 - "sinowealth.rs"
Cohesion: 0.05
Nodes (51): ACTIVE_DPI_GET, ACTIVE_DPI_SET, BATTERY, battery_report(), brightness_to_byte(), build_request(), builds_65_byte_request(), builds_dpi_color_request() (+43 more)

### Community 2 - "Result"
Cohesion: 0.10
Nodes (35): BatteryInfo, Connection, ConnectionKind, build_button_payload(), DpiInfo, ensure_success(), hid_to_polling_rate(), parse_active_dpi_stage() (+27 more)

### Community 3 - "devices.rs"
Cohesion: 0.09
Nodes (31): AsRef, classifies_model_i_2_as_pixart_scaffold(), Connection, DbError, default_candidates(), DEFAULT_RELATIVE_PATH, DeviceDatabase, DeviceEntry (+23 more)

### Community 4 - "manager.rs"
Cohesion: 0.10
Nodes (35): batteryinfo_as_battery, ApplyResult, base_view(), cache_key(), cache_key_is_stable(), database_has_model_o_wireless(), device_view_includes_profile_settings(), DeviceManager (+27 more)

### Community 5 - "package.json"
Cohesion: 0.06
Nodes (34): dependencies, @fontsource-variable/inter, lucide-svelte, description, devDependencies, svelte, svelte-check, @sveltejs/vite-plugin-svelte (+26 more)

### Community 6 - "mod.rs"
Cohesion: 0.06
Nodes (38): choose_endpoint(), ConnectionKind, DEFAULT_DELAY, DeviceStatus, Endpoint, endpoints_for(), enumerate(), HidError (+30 more)

### Community 7 - "actions.ts"
Cohesion: 0.07
Nodes (30): ALPHABET, bindingsToPinActions(), BUTTON_ID_TO_PIN, buttonActionToPinAction(), clamp(), DEFAULT_MAPPINGS, DEFAULT_PIN_ACTIONS, DOCK_ACTIONS (+22 more)

### Community 8 - "main.rs"
Cohesion: 0.14
Nodes (25): api, default_delay(), main(), open_browser(), Options, options_default_has_open_browser_enabled(), options_no_browser_flag_disables_browser(), options_open_flag_enables_browser() (+17 more)

### Community 9 - "extract_devices.py"
Cohesion: 0.09
Nodes (30): Any, argparse, datetime, build_render(), main(), Path, Build web-ready mouse renders from the extracted Glorious CORE assets. The…, Make the opaque white studio plate transparent, keeping the product. (+22 more)

### Community 11 - "glorious-service"
Cohesion: 0.15
Nodes (11): API, Build and run, DPI readback quirk, glorious-service, Layout, Permissions, Protocol notes (Sinowealth, Model O / D), Status (+3 more)

### Community 13 - "types.ts"
Cohesion: 0.08
Nodes (23): ActionCategory, ApplyResult, BatteryInfo, ButtonAction, ButtonBinding, ConnectionView, DeviceList, DeviceStatus (+15 more)

### Community 14 - "compilerOptions"
Cohesion: 0.13
Nodes (14): compilerOptions, allowJs, checkJs, isolatedModules, module, moduleDetection, moduleResolution, noEmit (+6 more)

### Community 16 - "api.ts"
Cohesion: 0.18
Nodes (10): API_BASE, ApiError, applySettings(), connectWs(), fetchDevice(), fetchDevices(), fetchHealth(), JSON_HEADERS (+2 more)

### Community 17 - "profiles.rs"
Cohesion: 0.09
Nodes (32): BTreeMap, buttonaction, DeviceProfileConfig, falls_back_to_defaults(), find_glorious_core_db(), import_glorious_core_db(), import_glorious_core_db_from_str(), imports_from_glorious_core_db_text() (+24 more)

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
Cohesion: 0.08
Nodes (29): BatteryInfo, ButtonAction, ButtonBinding, ConnectionView, DeviceStatusView, DeviceView, effect_name_for(), EffectName (+21 more)

### Community 23 - "AppState"
Cohesion: 0.40
Nodes (5): AppState, Instant, Sender, SharedManager, WsEvent

### Community 27 - "DeviceList"
Cohesion: 0.67
Nodes (3): DeviceList, DeviceView, Vec

## Knowledge Gaps
- **119 isolated node(s):** `TARGET_DONGLE`, `TARGET_MOUSE`, `CATEGORY_SYSTEM`, `CATEGORY_SENSOR`, `CATEGORY_RGB` (+114 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 246 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **13 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SinowealthDevice` connect `Result` to `sinowealth.rs`, `manager.rs`, `mod.rs`?**
  _High betweenness centrality (0.072) - this node is a cross-community bridge._
- **Why does `Response` connect `Result` to `sinowealth.rs`, `apply_device`, `api.rs`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Why does `DeviceManager` connect `manager.rs` to `main.rs`, `devices.rs`, `api.rs`?**
  _High betweenness centrality (0.049) - this node is a cross-community bridge._
- **What connects `TARGET_DONGLE`, `TARGET_MOUSE`, `CATEGORY_SYSTEM` to the rest of the system?**
  _119 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ws.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._
- **Should `sinowealth.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05388471177944862 - nodes in this community are weakly interconnected._
- **Should `Result` be split into smaller, more focused modules?**
  _Cohesion score 0.09931798806479114 - nodes in this community are weakly interconnected._
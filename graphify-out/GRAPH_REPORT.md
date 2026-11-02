# Graph Report - Glorious-Web-Launcher  (2026-11-03)

## Corpus Check
- 34 files · ~1,310,961 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 4 file(s) not represented in the graph (top: (none) 2, .css 1, .rules 1)

## Summary
- 690 nodes · 1386 edges · 46 communities (21 shown, 25 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4315861e`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- ws.rs
- sinowealth.rs
- HidError
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
- for
- types.rs
- AppState
- BatteryInfo
- Duration
- FirmwareInfo
- DeviceView
- HidApi
- HidDevice
- Error
- Mutex
- Path
- PathBuf
- DeviceStatus
- Display
- Duration
- Error
- FirmwareInfo
- Formatter
- From
- HidApi
- Mutex
- SettingsPatch
- Default
- RgbPatch

## God Nodes (most connected - your core abstractions)
1. `SinowealthDevice` - 34 edges
2. `DeviceManager` - 26 edges
3. `Response` - 24 edges
4. `DeviceEntry` - 20 edges
5. `ProfileStore` - 16 edges
6. `ensure_success()` - 16 edges
7. `ManagerError` - 15 edges
8. `AppState` - 15 edges
9. `HidError` - 15 edges
10. `Endpoint` - 14 edges

## Surprising Connections (you probably didn't know these)
- `import_glorious_core_db_from_str()` --calls--> `effect_name_for()`  [INFERRED]
  crates/glorious-service/src/profiles.rs → crates/glorious-service/src/types.rs
- `probe()` --references--> `DeviceManager`  [EXTRACTED]
  crates/glorious-service/src/main.rs → crates/glorious-service/src/manager.rs
- `run_server()` --references--> `DeviceManager`  [EXTRACTED]
  crates/glorious-service/src/main.rs → crates/glorious-service/src/manager.rs
- `ApiError` --references--> `ManagerError`  [EXTRACTED]
  crates/glorious-service/src/api.rs → crates/glorious-service/src/manager.rs
- `ProfileConfig` --references--> `LightingView`  [EXTRACTED]
  crates/glorious-service/src/profiles.rs → crates/glorious-service/src/types.rs

## Import Cycles
- 2-file cycle: `crates/glorious-service/src/api.rs -> crates/glorious-service/src/ws.rs -> crates/glorious-service/src/api.rs`

## Communities (46 total, 25 thin omitted)

### Community 0 - "ws.rs"
Cohesion: 0.15
Nodes (19): Box, broadcast, handle_socket(), DeviceView, Duration, IntoResponse, Result, State (+11 more)

### Community 1 - "sinowealth.rs"
Cohesion: 0.05
Nodes (51): ACTIVE_DPI_GET, ACTIVE_DPI_SET, BATTERY, battery_report(), brightness_to_byte(), build_request(), builds_65_byte_request(), builds_dpi_color_request() (+43 more)

### Community 2 - "HidError"
Cohesion: 0.12
Nodes (30): BatteryInfo, ConnectionKind, build_button_payload(), DpiInfo, ensure_success(), hid_to_polling_rate(), parse_active_dpi_stage(), parse_battery() (+22 more)

### Community 3 - "devices.rs"
Cohesion: 0.09
Nodes (30): AsRef, classifies_model_i_2_as_pixart_scaffold(), Connection, DbError, default_candidates(), DEFAULT_RELATIVE_PATH, DeviceDatabase, DeviceEntry (+22 more)

### Community 4 - "manager.rs"
Cohesion: 0.10
Nodes (35): batteryinfo_as_battery, Connection, ApplyResult, base_view(), cache_key(), cache_key_is_stable(), database_has_model_o_wireless(), Detected (+27 more)

### Community 5 - "package.json"
Cohesion: 0.06
Nodes (34): dependencies, @fontsource-variable/inter, lucide-svelte, description, devDependencies, svelte, svelte-check, @sveltejs/vite-plugin-svelte (+26 more)

### Community 6 - "mod.rs"
Cohesion: 0.06
Nodes (39): choose_endpoint(), ConnectionKind, DEFAULT_DELAY, DeviceStatus, Endpoint, endpoints_for(), enumerate(), HidError (+31 more)

### Community 7 - "actions.ts"
Cohesion: 0.08
Nodes (28): ALPHABET, BUTTON_ID_TO_PIN, clamp(), DEFAULT_MAPPINGS, DEFAULT_PIN_ACTIONS, DOCK_ACTIONS, DOCK_CATEGORIES, DPI_COLORS (+20 more)

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
Nodes (23): ApplyResult, BatteryInfo, ButtonAction, ButtonBinding, ConnectionView, DeviceList, DeviceStatus, DeviceView (+15 more)

### Community 14 - "compilerOptions"
Cohesion: 0.13
Nodes (14): compilerOptions, allowJs, checkJs, isolatedModules, module, moduleDetection, moduleResolution, noEmit (+6 more)

### Community 16 - "api.ts"
Cohesion: 0.18
Nodes (10): API_BASE, ApiError, applySettings(), connectWs(), fetchDevice(), fetchDevices(), fetchHealth(), JSON_HEADERS (+2 more)

### Community 17 - "profiles.rs"
Cohesion: 0.09
Nodes (36): BTreeMap, buttonaction, DeviceProfileConfig, falls_back_to_defaults(), find_glorious_core_db(), import_glorious_core_db(), import_glorious_core_db_from_str(), imports_from_glorious_core_db_text() (+28 more)

### Community 18 - "apply_device"
Cohesion: 0.15
Nodes (20): ApiError, ApplyResult, apply_device(), device_details(), DeviceList, health(), list_devices(), DeviceView (+12 more)

### Community 19 - "api.rs"
Cohesion: 0.15
Nodes (15): axum, corslayer, DEFAULT_BIND, EmbeddedAssets, router(), Arc, static_known_asset_served_with_correct_mime(), static_root_serves_index_html() (+7 more)

### Community 20 - "ApiError"
Cohesion: 0.29
Nodes (6): ApiError, From, IntoResponse, Self, JoinError, StatusCode

### Community 21 - "for"
Cohesion: 0.33
Nodes (6): for(), bindingsToPinActions(), buttonActionToPinAction(), draftFromDevice(), draftFromSettings(), rgbToHex()

### Community 22 - "types.rs"
Cohesion: 0.09
Nodes (28): BatteryInfo, ButtonAction, ButtonBinding, ConnectionView, DeviceStatusView, DeviceView, effect_name_for(), EffectName (+20 more)

### Community 23 - "AppState"
Cohesion: 0.40
Nodes (5): AppState, Instant, Sender, SharedManager, WsEvent

## Knowledge Gaps
- **119 isolated node(s):** `LAUNCHER_VERSION`, `PIN_SPECS`, `DEFAULT_MAPPINGS`, `DEFAULT_PIN_ACTIONS`, `PIN_LABELS` (+114 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 255 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **25 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SinowealthDevice` connect `HidError` to `sinowealth.rs`, `manager.rs`, `mod.rs`?**
  _High betweenness centrality (0.058) - this node is a cross-community bridge._
- **Why does `Response` connect `HidError` to `sinowealth.rs`, `apply_device`, `api.rs`?**
  _High betweenness centrality (0.045) - this node is a cross-community bridge._
- **Why does `DeviceManager` connect `manager.rs` to `main.rs`, `profiles.rs`, `HidError`, `api.rs`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **What connects `LAUNCHER_VERSION`, `PIN_SPECS`, `DEFAULT_MAPPINGS` to the rest of the system?**
  _119 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `sinowealth.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05388471177944862 - nodes in this community are weakly interconnected._
- **Should `HidError` be split into smaller, more focused modules?**
  _Cohesion score 0.11623143881208398 - nodes in this community are weakly interconnected._
- **Should `devices.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09200603318250378 - nodes in this community are weakly interconnected._
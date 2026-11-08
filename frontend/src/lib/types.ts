export type DeviceStatus =
  | 'connected'
  | 'asleep'
  | 'unsupported'
  | 'error'
  | 'disconnected';

export interface BatteryInfo {
  percentage: number;
  charging: boolean;
  status: string;
}

export interface FirmwareInfo {
  dongle?: string;
  mouse?: string;
  display: string;
}

export interface DpiStage {
  value: number;
  color?: string;
  x?: number;
  y?: number;
}

export interface SettingsView {
  active_profile: number;
  dpi_stage_count: number;
  dpi_stages: number[];
  dpi_colors?: string[];
  active_dpi_stage: number;
  polling_rate: number;
  debounce_ms: number;
  lod: number;
  sleep_seconds: number | null;
  buttons?: ButtonBinding[];
  lighting?: {
    effect: EffectId | number;
    effect_name?: string;
    brightness: number;
    rate: number;
    color: [number, number, number];
  } | null;

  /* ── Keychron Launcher parity (advanced / angle tabs) ── */
  xy_split?: boolean;
  competitive_mode?: boolean;
  angle_snap?: boolean;
  ripple_control?: boolean;
  motion_sync?: boolean;
  sensor_angle?: number;
  left_click_lock?: boolean;
  scroll_direction?: ScrollDirection;
  gaming_turbo?: GamingTurbo;
}

export interface LightingSettings {
  effect: number;
  brightness: number;
  rate: number;
  color: string;
}

export type ScrollDirection = 'standard' | 'reverse';

/** Gaming Turbo (rapid fire) binding: how often and how many repeats a key fires. */
export interface GamingTurbo {
  key: string;
  count: number;
  rate_ms: number;
}

export interface SettingsDraft {
  profile: number;
  dpi_stages: DpiStage[];
  active_dpi_stage: number;
  polling_rate: number;
  debounce_ms: number;
  lod: number;
  sleep_seconds: number | 'off';
  lighting: LightingSettings;

  /* ── Keychron Launcher parity ── */
  /** Split X/Y axis sensitivity (Advanced tab). */
  xy_split: boolean;
  /** Competitive mode: locks risky bindings while gaming (Advanced tab). */
  competitive_mode: boolean;
  /** Snap pointer movement to fixed angles in increments. */
  angle_snap: boolean;
  /** Ripple control for click/scroll repeat flood protection. */
  ripple_control: boolean;
  /** Motion sync keeps sensor sampling aligned with report intervals. */
  motion_sync: boolean;
  /** Sensor angle in degrees, -10..10 (Angle tab). */
  sensor_angle: number;
  /** Keep the left click usable even when the profile disables it. */
  left_click_lock: boolean;
  scroll_direction: ScrollDirection;
  /** Null/undefined when Gaming Turbo is not configured. */
  gaming_turbo?: GamingTurbo;
}

export interface ConnectionView {
  type: string;
  vid: string;
  pid: string;
  interface: number;
  usage_page: string;
  usage: string;
  path: string;
}

export interface DeviceCapabilities {
  maxProfiles: number;
  maxDpiStages: number;
  minDpi: number;
  maxDpi: number;
  pollingRates: number[];
  supportedLods: number[];
  hasAngleSnapping: boolean;
  hasRippleControl: boolean;
  hasMotionSync: boolean;
  hasSensorAngle: boolean;
  hasCompetitiveMode: boolean;
  hasXySplitDpi: boolean;
  hasGamingTurbo: boolean;
  hasLeftClickLock: boolean;
}

export interface DeviceView {
  slug: string;
  name: string;
  id: string;
  routerID: string;
  protocol: string;
  supported: boolean;
  connected: boolean;
  status: DeviceStatus;
  connection?: ConnectionView;
  image?: string;
  battery?: BatteryInfo;
  firmware?: FirmwareInfo;
  settings?: SettingsView;
  profiles?: Record<number, SettingsView>;
  error?: string;
  last_seen?: number;
}

export interface DeviceList {
  count: number;
  timestamp: number;
  devices: DeviceView[];
}

export type RgbEffectName =
  | 'off'
  | 'glorious'
  | 'seamless_breathing'
  | 'breathing'
  | 'static'
  | 'single_breathing'
  | 'tail'
  | 'rave'
  | 'wave';

export type EffectId = number;
export type EffectName = RgbEffectName;

export interface RgbPatch {
  brightness: number;
  wireless_brightness?: number;
  effect: EffectId | RgbEffectName | number;
  effect_name?: string;
  rate: number;
  color: [number, number, number];
}

export type ButtonAction =
  | { type: 'mouse'; button: number }
  | { type: 'keyboard'; key_code: number; modifiers?: number }
  | { type: 'dpi'; action: number }
  | { type: 'profile'; action: number }
  | { type: 'media'; key: number }
  | { type: 'macro'; macro_id: number; repeat_mode?: number; loop_count?: number }
  | { type: 'disabled' };

export interface ButtonBinding {
  pin: string;
  button_id: number;
  action: ButtonAction;
}

export interface MacroItem {
  id: number;
  name?: string;
  repeat_mode?: number;
  loop_count?: number;
  raw_bytes: number[];
}

export interface SettingsPatch {
  profile?: number;
  dpi_stages?: number[];
  dpi_colors?: string[];
  active_dpi_stage?: number;
  polling_rate?: number;
  debounce_ms?: number;
  lod?: number;
  sleep_seconds?: number;
  buttons?: ButtonBinding[];
  rgb?: RgbPatch;
  macros?: MacroItem[];

  /* ── Keychron Launcher parity (advanced / angle tabs) ── */
  xy_split?: boolean;
  competitive_mode?: boolean;
  angle_snap?: boolean;
  ripple_control?: boolean;
  motion_sync?: boolean;
  /** Sensor angle in degrees, clamped to -10..10. */
  sensor_angle?: number;
  left_click_lock?: boolean;
  scroll_direction?: ScrollDirection;
  gaming_turbo?: GamingTurbo;
}

export interface ApplyResult {
  slug: string;
  applied: string[];
  device: DeviceView;
}

export interface HealthInfo {
  status: string;
  service: string;
  version: string;
  uptime_seconds: number;
  ws_subscribers: number;
}

export type WsEvent =
  | { type: 'snapshot'; ts: number; devices: DeviceView[] }
  | { type: 'telemetry'; ts: number; devices: DeviceView[] }
  | {
      type: 'applied';
      ts: number;
      slug: string;
      applied: string[];
      device: DeviceView;
    }
  | { type: 'error'; ts: number; message: string };

export type ViewId =
  | 'customize'
  | 'pointer'
  | 'lighting'
  | 'advanced'
  | 'angle'
  | 'macro'
  | 'firmware';

export type TabId =
  | 'customize'
  | 'pointer'
  | 'lighting'
  | 'advanced'
  | 'angle'
  | 'macro'
  | 'firmware';

export type ActionCategory =
  | 'mousekeys'
  | 'basickeys'
  | 'shortcuts'
  | 'combinations'
  | 'multimedia'
  | 'macro'
  | 'dpi'
  | 'polling'
  | 'gaming'
  | 'profile'
  | 'disable';

export interface DockCategory {
  id: ActionCategory;
  label: string;
}

export interface DockAction {
  id: string;
  label: string;
}

export interface PinAction {
  pin: PinId;
  category: ActionCategory;
  action: string;
}

export type PinId =
  | 'left'
  | 'right'
  | 'middle'
  | 'scroll_up'
  | 'scroll_down'
  | 'forward'
  | 'backward'
  | 'dpi';

export type MacroEventType = 'down' | 'up' | 'delay' | 'text' | 'mouse';
export type MacroRepeatMode = 'no_repeat' | 'toggle' | 'while_holding';

export interface MacroEvent {
  id: string;
  type: MacroEventType;
  key?: string;
  code?: string;
  keyCode?: number;
  delay?: number;
  text?: string;
  mouseButton?: number;
}

export interface MacroDefinition {
  id: string;
  slot: number;
  name: string;
  repeatMode: MacroRepeatMode;
  loopCount: number;
  events: MacroEvent[];
}
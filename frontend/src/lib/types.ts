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
}

export interface LightingSettings {
  effect: number;
  brightness: number;
  rate: number;
  color: string;
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
  | { type: 'disabled' };

export interface ButtonBinding {
  pin: string;
  button_id: number;
  action: ButtonAction;
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
  | 'macro'
  | 'firmware';

export type TabId = Exclude<ViewId, 'macro'>;

export type ActionCategory =
  | 'mousekeys'
  | 'basickeys'
  | 'shortcuts'
  | 'multimedia'
  | 'macro'
  | 'dpi'
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

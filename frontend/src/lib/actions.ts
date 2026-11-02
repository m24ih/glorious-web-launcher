//! UI-only catalogs: stage pins, dock actions, lighting effects and small
//! formatting helpers shared by the components.

import type {
  ActionCategory,
  DockAction,
  DockCategory,
  DpiStage,
  EffectId,
  EffectName,
  PinAction,
  PinId,
  SettingsView,
  SettingsDraft,
} from './types';

export const LAUNCHER_VERSION = '0.1.0';

/* ── Stage pins ─────────────────────────────────────────────────────────── */

/** A hardware button as rendered on the stage, with callout geometry.
 * All coordinates are percentages of the stage canvas (8:5), where the
 * top-down mouse render is centered at (50, 50) and spans x 37-63, y 5-95. */
export interface PinSpec {
  id: PinId;
  label: string;
  side: 'left' | 'right';
  /** Pill center on the outer side. */
  pillX: number;
  pillY: number;
  /** Attachment point on the mouse body. */
  anchorX: number;
  anchorY: number;
}

export const PIN_SPECS: PinSpec[] = [
  {
    id: 'left',
    label: 'Left Click',
    side: 'left',
    pillX: 12,
    pillY: 20,
    anchorX: 45.6,
    anchorY: 16,
  },
  {
    id: 'forward',
    label: 'Forward',
    side: 'left',
    pillX: 14,
    pillY: 56,
    anchorX: 41.6,
    anchorY: 54,
  },
  {
    id: 'backward',
    label: 'Backward',
    side: 'left',
    pillX: 16,
    pillY: 68,
    anchorX: 41.8,
    anchorY: 65,
  },
  {
    id: 'right',
    label: 'Right Click',
    side: 'right',
    pillX: 88,
    pillY: 20,
    anchorX: 54.4,
    anchorY: 16,
  },
  {
    id: 'scroll_up',
    label: 'Scroll Up',
    side: 'right',
    pillX: 86,
    pillY: 38,
    anchorX: 50,
    anchorY: 34,
  },
  {
    id: 'middle',
    label: 'Middle Click',
    side: 'right',
    pillX: 84,
    pillY: 48,
    anchorX: 50.2,
    anchorY: 41,
  },
  {
    id: 'scroll_down',
    label: 'Scroll Down',
    side: 'right',
    pillX: 84,
    pillY: 58,
    anchorX: 50,
    anchorY: 47,
  },
  {
    id: 'dpi',
    label: 'DPI Loop',
    side: 'right',
    pillX: 85,
    pillY: 76,
    anchorX: 44.8,
    anchorY: 72,
  },
];

export const DEFAULT_MAPPINGS: PinAction[] = [
  { pin: 'left', category: 'basickeys', action: 'Left Click' },
  { pin: 'right', category: 'basickeys', action: 'Right Click' },
  { pin: 'middle', category: 'basickeys', action: 'Middle Click' },
  { pin: 'scroll_up', category: 'basickeys', action: 'Scroll Up' },
  { pin: 'scroll_down', category: 'basickeys', action: 'Scroll Down' },
  { pin: 'dpi', category: 'dpi', action: 'DPI Loop' },
  { pin: 'forward', category: 'basickeys', action: 'Forward' },
  { pin: 'backward', category: 'basickeys', action: 'Backward' },
];

export const DEFAULT_PIN_ACTIONS = DEFAULT_MAPPINGS;

export const PIN_LABELS: Record<PinId, string> = Object.fromEntries(
  PIN_SPECS.map((p) => [p.id, p.label]),
) as Record<PinId, string>;

/* ── Dock ───────────────────────────────────────────────────────────────── */

export const DOCK_CATEGORIES: DockCategory[] = [
  { id: 'basickeys', label: 'Basic Keys' },
  { id: 'shortcuts', label: 'Shortcut Keys' },
  { id: 'multimedia', label: 'Multimedia Keys' },
  { id: 'macro', label: 'Macro' },
  { id: 'dpi', label: 'DPI' },
  { id: 'disable', label: 'Disable' },
];

const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');

export const DOCK_ACTIONS: Record<ActionCategory, DockAction[]> = {
  basickeys: [
    ...ALPHABET.map((k) => ({ id: `key:${k}`, label: k })),
    { id: 'key:esc', label: 'Esc' },
    { id: 'key:tab', label: 'Tab' },
    { id: 'key:capslock', label: 'Caps Lock' },
    { id: 'key:space', label: 'Space' },
    { id: 'key:enter', label: 'Enter' },
    { id: 'key:backspace', label: 'Backspace' },
    { id: 'key:shift', label: 'Shift' },
    { id: 'key:ctrl', label: 'Ctrl' },
    { id: 'key:alt', label: 'Alt' },
    { id: 'key:win', label: 'Win' },
  ],
  shortcuts: [
    { id: 'sc:ctrl+c', label: 'Ctrl + C' },
    { id: 'sc:ctrl+v', label: 'Ctrl + V' },
    { id: 'sc:ctrl+x', label: 'Ctrl + X' },
    { id: 'sc:ctrl+z', label: 'Ctrl + Z' },
    { id: 'sc:ctrl+y', label: 'Ctrl + Y' },
    { id: 'sc:ctrl+a', label: 'Ctrl + A' },
    { id: 'sc:ctrl+s', label: 'Ctrl + S' },
    { id: 'sc:ctrl+f', label: 'Ctrl + F' },
    { id: 'sc:ctrl+w', label: 'Ctrl + W' },
    { id: 'sc:ctrl+t', label: 'Ctrl + T' },
    { id: 'sc:alt+tab', label: 'Alt + Tab' },
    { id: 'sc:ctrl+alt+del', label: 'Ctrl + Alt + Del' },
    { id: 'sc:win+d', label: 'Win + D' },
    { id: 'sc:win+l', label: 'Win + L' },
    { id: 'sc:win+s', label: 'Win + S' },
    { id: 'sc:ctrl+shift+esc', label: 'Ctrl + Shift + Esc' },
    { id: 'sc:win+tab', label: 'Win + Tab' },
    { id: 'sc:alt+f4', label: 'Alt + F4' },
  ],
  multimedia: [
    { id: 'mm:play', label: 'Play / Pause' },
    { id: 'mm:next', label: 'Next Track' },
    { id: 'mm:prev', label: 'Prev Track' },
    { id: 'mm:stop', label: 'Stop' },
    { id: 'mm:volup', label: 'Volume Up' },
    { id: 'mm:voldown', label: 'Volume Down' },
    { id: 'mm:mute', label: 'Mute' },
    { id: 'mm:calc', label: 'Calculator' },
    { id: 'mm:mail', label: 'Mail' },
    { id: 'mm:home', label: 'Browser Home' },
    { id: 'mm:refresh', label: 'Browser Refresh' },
    { id: 'mm:brightness_up', label: 'Brightness Up' },
    { id: 'mm:brightness_down', label: 'Brightness Down' },
    { id: 'mm:screen_lock', label: 'Screen Lock' },
    { id: 'mm:my_computer', label: 'My Computer' },
  ],
  macro: [
    { id: 'macro:m1', label: 'Macro 1' },
    { id: 'macro:m2', label: 'Macro 2' },
    { id: 'macro:m3', label: 'Macro 3' },
    { id: 'macro:m4', label: 'Macro 4' },
    { id: 'macro:m5', label: 'Macro 5' },
    { id: 'macro:m6', label: 'Macro 6' },
  ],
  dpi: [
    { id: 'dpi:400', label: '400 DPI' },
    { id: 'dpi:800', label: '800 DPI' },
    { id: 'dpi:1200', label: '1200 DPI' },
    { id: 'dpi:1600', label: '1600 DPI' },
    { id: 'dpi:3200', label: '3200 DPI' },
    { id: 'dpi:6400', label: '6400 DPI' },
    { id: 'dpi:cycle', label: 'DPI Loop' },
    { id: 'dpi:up', label: 'DPI Up' },
    { id: 'dpi:down', label: 'DPI Down' },
  ],
  disable: [{ id: 'disable', label: 'Disabled' }],
};

/* ── Lighting ───────────────────────────────────────────────────────────── */

export interface EffectSpec {
  id: EffectId;
  name: EffectName;
  label: string;
  /** Whether this effect uses the color picker. */
  colored: boolean;
  /** Optional CSS gradient preview of the effect. */
  preview: string;
}

export const EFFECTS: EffectSpec[] = [
  {
    id: 1,
    name: 'glorious',
    label: 'Glorious Mode',
    colored: false,
    preview: 'linear-gradient(135deg, #ff2d2d, #ffb800, #2dff6a, #2d8cff, #c12dff)',
  },
  {
    id: 2,
    name: 'seamless_breathing',
    label: 'Seamless Breathing',
    colored: false,
    preview: 'linear-gradient(135deg, #ff2d2d22, #ffb80022, #2d8cff22, #ff2d2d55)',
  },
  {
    id: 3,
    name: 'breathing',
    label: 'Breathing',
    colored: true,
    preview: '',
  },
  {
    id: 4,
    name: 'static',
    label: 'Single Color',
    colored: true,
    preview: '',
  },
  {
    id: 5,
    name: 'single_breathing',
    label: 'Single Breathing',
    colored: true,
    preview: '',
  },
  {
    id: 6,
    name: 'tail',
    label: 'Tail',
    colored: true,
    preview: 'linear-gradient(90deg, #ffb800, #ffb80022)',
  },
  {
    id: 7,
    name: 'rave',
    label: 'Rave',
    colored: false,
    preview: 'linear-gradient(90deg, #ff2d2d, #ffb800, #2dff6a)',
  },
  {
    id: 8,
    name: 'wave',
    label: 'Wave',
    colored: false,
    preview: 'linear-gradient(90deg, #ff2d2d, #ffb800, #2d8cff, #c12dff)',
  },
  {
    id: 0,
    name: 'off',
    label: 'Off',
    colored: false,
    preview: '',
  },
];

export const SLEEP_PRESETS: Array<{ label: string; seconds: number | 'off' }> = [
  { label: '1 min', seconds: 60 },
  { label: '2 min', seconds: 120 },
  { label: '3 min', seconds: 180 },
  { label: '5 min', seconds: 300 },
  { label: '10 min', seconds: 600 },
  { label: '15 min', seconds: 900 },
];

export const LIGHTING_EFFECTS = EFFECTS;
export const SLEEP_OPTIONS = [...SLEEP_PRESETS, { label: 'Off', seconds: 'off' as const }];

/* ── Small helpers ──────────────────────────────────────────────────────── */

export function rgbToHex(rgb: [number, number, number] | number[]): string {
  const [r = 0, g = 0, b = 0] = rgb;
  const toHex = (n: number) =>
    Math.min(255, Math.max(0, Math.round(n)))
      .toString(16)
      .padStart(2, '0');
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

export function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace('#', '');
  const full =
    clean.length === 3
      ? clean
          .split('')
          .map((c) => c + c)
          .join('')
      : clean;
  const n = parseInt(full || 'ffb800', 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function clamp(v: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, v));
}

/** Percentage for `input[type=range]` fill gradients. */
export function rangeFill(v: number, min: number, max: number): string {
  return `${(clamp(v, min, max) / (max - min)) * 100}%`;
}

/** Which DPI color badge a stage gets (Glorious CORE preset colors). */
export const DPI_COLORS = [
  '#FFFF00',
  '#0000FF',
  '#FF0000',
  '#00FF00',
  '#00FFFF',
  '#FF00FF',
  '#FF8000',
  '#C12DFF',
];

export function dpiColor(index: number): string {
  return DPI_COLORS[index % DPI_COLORS.length];
}

/** Seed a settings draft from read-back hardware values. */
export function draftFromDevice(device?: { settings?: SettingsView | null } | null): SettingsDraft {
  const settings = device?.settings;

  let stages: DpiStage[];
  if (settings && settings.dpi_stages && settings.dpi_stages.length > 0) {
    stages = settings.dpi_stages.slice(0, 8).map((value, i) => ({
      value,
      color: settings.dpi_colors?.[i] || DPI_COLORS[i % DPI_COLORS.length],
    }));
  } else {
    stages = [
      { value: 400, color: DPI_COLORS[0] },
      { value: 800, color: DPI_COLORS[1] },
      { value: 1600, color: DPI_COLORS[2] },
      { value: 3200, color: DPI_COLORS[3] },
    ];
  }

  let lighting = {
    effect: 1,
    brightness: 100,
    rate: 50,
    color: '#ffb800',
  };

  if (settings?.lighting) {
    lighting = {
      effect: settings.lighting.effect as EffectId,
      brightness: settings.lighting.brightness,
      rate: settings.lighting.rate,
      color: rgbToHex(settings.lighting.color),
    };
  }

  return {
    profile: settings?.active_profile ?? 1,
    dpi_stages: stages,
    active_dpi_stage: settings?.active_dpi_stage ?? 1,
    polling_rate: settings?.polling_rate ?? 1000,
    debounce_ms: settings?.debounce_ms ?? 10,
    lod: settings?.lod ?? 1,
    sleep_seconds: settings?.sleep_seconds ?? 'off',
    lighting,
  };
}

export const draftFromSettings = (settings?: SettingsView | null) =>
  draftFromDevice(settings ? { settings } : null);

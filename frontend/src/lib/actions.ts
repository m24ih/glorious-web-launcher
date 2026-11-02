//! UI-only catalogs: stage pins, dock actions, lighting effects and small
//! formatting helpers shared by the components.

import type {
  ActionCategory,
  ButtonAction,
  ButtonBinding,
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
  { pin: 'left', category: 'mousekeys', action: 'Left Click' },
  { pin: 'right', category: 'mousekeys', action: 'Right Click' },
  { pin: 'middle', category: 'mousekeys', action: 'Middle Click' },
  { pin: 'scroll_up', category: 'mousekeys', action: 'Scroll Up' },
  { pin: 'scroll_down', category: 'mousekeys', action: 'Scroll Down' },
  { pin: 'dpi', category: 'dpi', action: 'DPI Loop' },
  { pin: 'forward', category: 'mousekeys', action: 'Forward' },
  { pin: 'backward', category: 'mousekeys', action: 'Backward' },
];

export const DEFAULT_PIN_ACTIONS = DEFAULT_MAPPINGS;

export const PIN_LABELS: Record<PinId, string> = Object.fromEntries(
  PIN_SPECS.map((p) => [p.id, p.label]),
) as Record<PinId, string>;

/* ── Dock ───────────────────────────────────────────────────────────────── */

export const DOCK_CATEGORIES: DockCategory[] = [
  { id: 'mousekeys', label: 'Mouse Keys' },
  { id: 'basickeys', label: 'Basic Keys' },
  { id: 'shortcuts', label: 'Shortcut Keys' },
  { id: 'multimedia', label: 'Multimedia Keys' },
  { id: 'macro', label: 'Macro' },
  { id: 'dpi', label: 'DPI' },
  { id: 'disable', label: 'Disable' },
];

const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');

export const DOCK_ACTIONS: Record<ActionCategory, DockAction[]> = {
  mousekeys: [
    { id: 'mouse:left', label: 'Left Click' },
    { id: 'mouse:right', label: 'Right Click' },
    { id: 'mouse:middle', label: 'Middle Click' },
    { id: 'mouse:forward', label: 'Forward' },
    { id: 'mouse:backward', label: 'Backward' },
    { id: 'mouse:scroll_up', label: 'Scroll Up' },
    { id: 'mouse:scroll_down', label: 'Scroll Down' },
    { id: 'mouse:profile_cycle', label: 'Profile Cycle' },
    { id: 'mouse:profile_up', label: 'Profile Up' },
    { id: 'mouse:profile_down', label: 'Profile Down' },
    { id: 'mouse:battery', label: 'Battery Check' },
    { id: 'dpi:loop', label: 'DPI Loop' },
  ],
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
    colored: false,
    preview: 'linear-gradient(180deg, #fff60a, #0aff7d, #ff600a, #0affe5, #ff0ad4, #ff0000)',
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
    preview: 'linear-gradient(180deg, #ff2d2d, #ffb800, #2d8cff, #c12dff)',
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

/* ── Button Bindings & Hardware Protocol Mappings ───────────────────────── */

export const PIN_TO_BUTTON_ID: Record<PinId, number> = {
  left: 1,
  right: 2,
  middle: 3,
  backward: 4,
  forward: 5,
  scroll_up: 0x10, // 16
  scroll_down: 0x11, // 17
  dpi: 0x14, // 20
};

export const BUTTON_ID_TO_PIN: Record<number, PinId> = {
  1: 'left',
  2: 'right',
  3: 'middle',
  4: 'backward',
  5: 'forward',
  0x10: 'scroll_up',
  0x11: 'scroll_down',
  0x14: 'dpi',
};

export const MULTIMEDIA_KEY_CODES: Record<string, number> = {
  'Play / Pause': 0x00cd,
  'Mute': 0x00e2,
  'Volume Up': 0x00e9,
  'Volume Down': 0x00ea,
  'Next Track': 0x00b5,
  'Prev Track': 0x00b6,
  'Previous Track': 0x00b6,
  'Stop': 0x00b7,
  'Calculator': 0x0192,
  'Mail': 0x018a,
  'Browser Home': 0x0223,
  'Browser Refresh': 0x0227,
  'Brightness Up': 0x006f,
  'Brightness Down': 0x0070,
  'Screen Lock': 0x019e,
  'My Computer': 0x0194,
};

export const SHORTCUT_DEFINITIONS: Record<string, { key_code: number; modifiers: number }> = {
  'Ctrl + C': { key_code: 0x06, modifiers: 1 },
  'Ctrl + V': { key_code: 0x19, modifiers: 1 },
  'Ctrl + X': { key_code: 0x1B, modifiers: 1 },
  'Ctrl + Z': { key_code: 0x1D, modifiers: 1 },
  'Ctrl + Y': { key_code: 0x1C, modifiers: 1 },
  'Ctrl + A': { key_code: 0x04, modifiers: 1 },
  'Ctrl + S': { key_code: 0x16, modifiers: 1 },
  'Ctrl + F': { key_code: 0x09, modifiers: 1 },
  'Ctrl + W': { key_code: 0x1A, modifiers: 1 },
  'Ctrl + T': { key_code: 0x17, modifiers: 1 },
  'Alt + Tab': { key_code: 0x2B, modifiers: 4 },
  'Ctrl + Alt + Del': { key_code: 0x4C, modifiers: 5 },
  'Win + D': { key_code: 0x07, modifiers: 8 },
  'Win + L': { key_code: 0x0F, modifiers: 8 },
  'Win + S': { key_code: 0x16, modifiers: 8 },
  'Ctrl + Shift + Esc': { key_code: 0x29, modifiers: 3 },
  'Win + Tab': { key_code: 0x2B, modifiers: 8 },
  'Alt + F4': { key_code: 0x3D, modifiers: 4 },
};

export const SPECIAL_KEY_CODES: Record<string, { key_code: number; modifiers?: number }> = {
  'Esc': { key_code: 0x29 },
  'Tab': { key_code: 0x2B },
  'Caps Lock': { key_code: 0x39 },
  'Space': { key_code: 0x2C },
  'Enter': { key_code: 0x28 },
  'Backspace': { key_code: 0x2A },
  'Shift': { key_code: 0xE1, modifiers: 2 },
  'Ctrl': { key_code: 0xE0, modifiers: 1 },
  'Alt': { key_code: 0xE2, modifiers: 4 },
  'Win': { key_code: 0xE3, modifiers: 8 },
};

export function pinActionToButtonAction(actionStr: string, category?: ActionCategory): ButtonAction {
  const norm = actionStr.trim();

  // Mouse keys category & basic mouse actions
  if (norm === 'Left Click' || norm === 'mouse:left') return { type: 'mouse', button: 1 };
  if (norm === 'Right Click' || norm === 'mouse:right') return { type: 'mouse', button: 2 };
  if (norm === 'Middle Click' || norm === 'mouse:middle') return { type: 'mouse', button: 3 };
  if (norm === 'Backward' || norm === 'mouse:backward') return { type: 'mouse', button: 4 };
  if (norm === 'Forward' || norm === 'mouse:forward') return { type: 'mouse', button: 5 };
  if (norm === 'Scroll Up' || norm === 'mouse:scroll_up') return { type: 'mouse', button: 0x10 };
  if (norm === 'Scroll Down' || norm === 'mouse:scroll_down') return { type: 'mouse', button: 0x11 };

  // Profile actions & Battery check
  if (
    norm === 'Profile Cycle' ||
    norm === 'mouse:profile_cycle' ||
    norm === 'Profile Up' ||
    norm === 'mouse:profile_up'
  ) {
    return { type: 'profile', action: 4 };
  }
  if (norm === 'Profile Down' || norm === 'mouse:profile_down') {
    return { type: 'profile', action: 3 };
  }
  if (norm === 'Battery Check' || norm === 'mouse:battery') {
    return { type: 'mouse', button: 0x0C };
  }

  // Disabled
  if (norm === 'Disabled' || norm === 'disable' || category === 'disable') return { type: 'disabled' };

  // DPI
  if (norm === 'DPI Loop' || norm === 'dpi:cycle' || norm === 'dpi:loop') return { type: 'dpi', action: 1 };
  if (norm === 'DPI Up' || norm === 'dpi:up') return { type: 'dpi', action: 2 };
  if (norm === 'DPI Down' || norm === 'dpi:down') return { type: 'dpi', action: 3 };
  if (category === 'dpi' || (category === 'mousekeys' && (norm === 'DPI Loop' || norm === 'dpi:loop'))) {
    return { type: 'dpi', action: 1 };
  }

  // Multimedia keys
  if (MULTIMEDIA_KEY_CODES[norm] !== undefined) {
    return { type: 'media', key: MULTIMEDIA_KEY_CODES[norm] };
  }

  // Predefined shortcuts
  if (SHORTCUT_DEFINITIONS[norm]) {
    return { type: 'keyboard', ...SHORTCUT_DEFINITIONS[norm] };
  }

  // Dynamic compound shortcuts (e.g. "Ctrl + Shift + A")
  if (norm.includes('+')) {
    const parts = norm.split('+').map((p) => p.trim());
    let modifiers = 0;
    let keyPart = '';
    for (const part of parts) {
      const lower = part.toLowerCase();
      if (lower === 'ctrl') modifiers |= 1;
      else if (lower === 'shift') modifiers |= 2;
      else if (lower === 'alt') modifiers |= 4;
      else if (lower === 'win' || lower === 'cmd' || lower === 'gui') modifiers |= 8;
      else keyPart = part;
    }
    let key_code = 0;
    if (keyPart.length === 1 && keyPart >= 'A' && keyPart <= 'Z') {
      key_code = keyPart.charCodeAt(0) - 65 + 0x04;
    } else if (keyPart.length === 1 && keyPart >= 'a' && keyPart <= 'z') {
      key_code = keyPart.toUpperCase().charCodeAt(0) - 65 + 0x04;
    } else if (keyPart === 'Esc') key_code = 0x29;
    else if (keyPart === 'Tab') key_code = 0x2B;
    else if (keyPart === 'Del' || keyPart === 'Delete') key_code = 0x4C;
    else if (/^F[1-9]|1[0-2]$/.test(keyPart)) {
      const fn = parseInt(keyPart.slice(1), 10);
      key_code = 0x3a + (fn - 1);
    }
    if (key_code > 0) {
      return { type: 'keyboard', key_code, modifiers };
    }
  }

  // Keyboard letters 'A'..'Z'
  if (norm.length === 1 && norm >= 'A' && norm <= 'Z') {
    return { type: 'keyboard', key_code: norm.charCodeAt(0) - 65 + 0x04 };
  }
  if (norm.length === 1 && norm >= 'a' && norm <= 'z') {
    return { type: 'keyboard', key_code: norm.toUpperCase().charCodeAt(0) - 65 + 0x04 };
  }

  // Special keys
  if (SPECIAL_KEY_CODES[norm]) {
    return { type: 'keyboard', ...SPECIAL_KEY_CODES[norm] };
  }

  return { type: 'disabled' };
}

export function buttonActionToPinAction(action: ButtonAction, pin: PinId): PinAction {
  switch (action.type) {
    case 'mouse': {
      let label = 'Left Click';
      if (action.button === 1) label = 'Left Click';
      else if (action.button === 2) label = 'Right Click';
      else if (action.button === 3) label = 'Middle Click';
      else if (action.button === 4) label = 'Backward';
      else if (action.button === 5) label = 'Forward';
      else if (action.button === 0x10) label = 'Scroll Up';
      else if (action.button === 0x11) label = 'Scroll Down';
      else if (action.button === 0x18) label = 'Profile Cycle';
      else if (action.button === 0x19) label = 'Profile Down';
      else if (action.button === 0x0C) label = 'Battery Check';
      return { pin, category: 'mousekeys', action: label };
    }
    case 'profile': {
      const label = action.action === 3 ? 'Profile Down' : 'Profile Cycle';
      return { pin, category: 'mousekeys', action: label };
    }
    case 'dpi': {
      let label = 'DPI Loop';
      if (action.action === 2) label = 'DPI Up';
      else if (action.action === 3) label = 'DPI Down';
      return { pin, category: 'dpi', action: label };
    }
    case 'media': {
      const entry = Object.entries(MULTIMEDIA_KEY_CODES).find(
        ([, code]) => code === action.key
      );
      const label = entry ? entry[0] : 'Play / Pause';
      return { pin, category: 'multimedia', action: label };
    }
    case 'disabled': {
      return { pin, category: 'disable', action: 'Disabled' };
    }
    case 'keyboard': {
      const mod = action.modifiers ?? 0;
      // Reverse check predefined shortcuts
      const shortcutMatch = Object.entries(SHORTCUT_DEFINITIONS).find(
        ([, def]) => def.key_code === action.key_code && def.modifiers === mod
      );
      if (shortcutMatch) {
        return { pin, category: 'shortcuts', action: shortcutMatch[0] };
      }

      // Check letters A-Z (if no modifiers)
      if (mod === 0 && action.key_code >= 0x04 && action.key_code <= 0x1d) {
        const letter = String.fromCharCode(65 + action.key_code - 0x04);
        return { pin, category: 'basickeys', action: letter };
      }

      // Check special keys
      const specialMatch = Object.entries(SPECIAL_KEY_CODES).find(
        ([, def]) => def.key_code === action.key_code && (def.modifiers ?? 0) === mod
      );
      if (specialMatch) {
        return { pin, category: 'basickeys', action: specialMatch[0] };
      }

      return { pin, category: 'basickeys', action: 'Space' };
    }
    default:
      return { pin, category: 'disable', action: 'Disabled' };
  }
}

/** Convert UI PinAction mappings to wire ButtonBinding array */
export function pinActionsToBindings(mappings: PinAction[]): ButtonBinding[] {
  return mappings.map((m) => {
    const button_id = PIN_TO_BUTTON_ID[m.pin] ?? 1;
    const action = pinActionToButtonAction(m.action, m.category);
    return {
      pin: m.pin,
      button_id,
      action,
    };
  });
}

/** Convert wire ButtonBinding array back to UI PinAction mappings */
export function bindingsToPinActions(bindings: ButtonBinding[]): PinAction[] {
  const result: Record<PinId, PinAction> = Object.fromEntries(
    DEFAULT_MAPPINGS.map((m) => [m.pin, { ...m }])
  ) as Record<PinId, PinAction>;

  for (const b of bindings) {
    const pin: PinId | undefined = (b.pin in PIN_TO_BUTTON_ID)
      ? (b.pin as PinId)
      : BUTTON_ID_TO_PIN[b.button_id];
    if (pin && b.action) {
      result[pin] = buttonActionToPinAction(b.action, pin);
    }
  }

  return Object.values(result);
}

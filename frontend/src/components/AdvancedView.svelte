<script lang="ts">
  import { Lock, Power, TriangleAlert, ShieldCheck } from 'lucide-svelte';
  import type { DeviceView, ScrollDirection, SettingsDraft } from '../lib/types';
  import { SLEEP_OPTIONS, clamp } from '../lib/actions';
  import { i18n } from '../lib/i18n.svelte';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
    onFactoryReset?: () => void;
    device?: DeviceView | null;
    caps?: import('../lib/types').DeviceCapabilities;
  }

  let { draft, onChange, onFactoryReset, device = null, caps }: Props = $props();

  const LOD_OPTIONS = [
    { value: 1, label: '1.0 mm', hintKey: 'advanced.lod_low_hint' },
    { value: 2, label: '2.0 mm', hintKey: 'advanced.lod_high_hint' },
  ] as const;

  const SCROLL_MODES: Array<{ value: ScrollDirection; labelKey: string; hintKey: string }> = [
    { value: 'standard', labelKey: 'advanced.scroll_standard', hintKey: 'advanced.scroll_standard_hint' },
    { value: 'reverse', labelKey: 'advanced.scroll_reverse', hintKey: 'advanced.scroll_reverse_hint' },
  ];

  let confirmReset = $state(false);

  const isConnected = $derived(
    device?.connected === true && device?.status === 'connected',
  );
  const deviceLabel = $derived(device?.name ?? i18n.t('status.connected'));
  const profileCount = $derived(
    device?.profiles ? Math.max(Object.keys(device.profiles).length, 1) : 3,
  );

  // Check whether any advanced sensor DSP features are actually supported on this hardware
  const hasAnySensorDsp = $derived(
    Boolean(caps?.hasAngleSnapping || caps?.hasRippleControl || caps?.hasMotionSync)
  );

  function formatSleep(value: number | 'off'): string {
    if (value === 'off' || value === 0) return i18n.t('lighting.sleep_off');
    if (value % 60 === 0) {
      const minutes = value / 60;
      return minutes >= 60 ? `${minutes / 60} hr` : `${minutes} min`;
    }
    return `${value} s`;
  }

  const sleepReadout = $derived(formatSleep(draft.sleep_seconds));

  function setDebounce(ms: number) {
    onChange((prev) => ({ ...prev, debounce_ms: clamp(ms, 0, 16) }));
  }

  function setLod(lod: number) {
    onChange((prev) => ({ ...prev, lod }));
  }

  function setScrollDirection(direction: ScrollDirection) {
    onChange((prev) => ({ ...prev, scroll_direction: direction }));
  }

  function setSleep(seconds: number | 'off') {
    onChange((prev) => ({ ...prev, sleep_seconds: seconds }));
  }

  function requestReset() {
    confirmReset = true;
  }

  function confirmFactoryReset() {
    confirmReset = false;
    onFactoryReset?.();
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === 'Escape') confirmReset = false;
  }}
/>

<div class="flex-1 overflow-y-auto p-6 lg:p-8">
  <div class="mx-auto max-w-[860px] space-y-6">
    <!-- Header -->
    <div class="flex flex-wrap items-end justify-between gap-4 border-b border-edge/60 pb-4">
      <div>
        <h2 class="text-lg font-bold tracking-tight text-ink">{i18n.t('advanced.title')}</h2>
        <p class="text-xs text-ink-dim">
          {i18n.t('advanced.subtitle')}
        </p>
      </div>
      <div
        class="flex items-center gap-2 rounded-full border border-edge bg-card px-3 py-1.5 text-[11px] text-ink-dim"
        title="Connected hardware"
      >
        <span
          class="h-2 w-2 rounded-full shadow-[0_0_8px_currentColor]"
          style="color: {isConnected ? '#11c468' : '#6b6b75'}; background: currentColor;"
        ></span>
        <span class="font-bold text-ink" translate="no">{deviceLabel}</span>
      </div>
    </div>

    <!-- 2-column grid for primary sensor switches -->
    <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
      <!-- Lift-Off Distance (LOD) -->
      <section class="flex flex-col justify-between rounded-2xl border border-edge bg-card p-5">
        <div>
          <div class="flex items-center justify-between">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              {i18n.t('advanced.lod_title')}
            </h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious">
              {draft.lod}.0 mm
            </span>
          </div>
          <p class="mt-1 text-xs text-ink-dim">
            {i18n.t('advanced.lod_desc')}
          </p>
        </div>

        <div class="mt-4 grid grid-cols-2 gap-3">
          {#each LOD_OPTIONS as opt (opt.value)}
            {@const isSelected = draft.lod === opt.value}
            <button
              type="button"
              onclick={() => setLod(opt.value)}
              aria-pressed={isSelected}
              class="flex flex-col items-start rounded-xl border p-3 text-left transition {isSelected
                ? 'border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/60 shadow-[0_0_14px_rgba(255,184,0,0.18)]'
                : 'border-edge bg-canvas/60 text-ink-dim hover:border-edge-bright hover:text-ink'}"
            >
              <span class="text-sm font-extrabold">{opt.label}</span>
              <span class="mt-0.5 text-[10px] text-ink-faint">{i18n.t(opt.hintKey)}</span>
            </button>
          {/each}
        </div>

        <p class="mt-3 text-[10px] text-ink-faint">
          <span class="text-emerald-400">✓</span> {i18n.t('advanced.lod_calibrated', { device: deviceLabel })}
        </p>
      </section>

      <!-- Debounce Time Slider -->
      <section class="flex flex-col justify-between rounded-2xl border border-edge bg-card p-5">
        <div>
          <div class="flex items-center justify-between">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              {i18n.t('advanced.debounce_title')}
            </h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious">
              {draft.debounce_ms} ms
            </span>
          </div>
          <p class="mt-1 text-xs text-ink-dim">
            {i18n.t('advanced.debounce_desc')}
          </p>
        </div>

        <div class="mt-4">
          <input
            type="range"
            min="0"
            max="16"
            step="1"
            value={draft.debounce_ms}
            style="--fill: {(draft.debounce_ms / 16) * 100}%"
            oninput={(e) => setDebounce(parseInt(e.currentTarget.value, 10))}
            class="w-full cursor-pointer"
          />
          <div class="mt-2 flex justify-between text-[10px] font-medium text-ink-faint">
            <span>{i18n.t('advanced.debounce_min')}</span>
            <span>{i18n.t('advanced.debounce_max')}</span>
          </div>
        </div>

        <p class="mt-3 text-[10px] text-ink-faint">
          {i18n.t('advanced.debounce_hint')}
        </p>
      </section>
    </div>

    <!-- Power Management (Standby Sleep Timer) -->
    <section class="rounded-2xl border border-edge bg-card p-5">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div>
          <div class="flex items-center gap-2">
            <Power size={14} class="text-glorious" />
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              {i18n.t('advanced.power_title')}
            </h3>
          </div>
          <p class="mt-1 text-xs text-ink-dim">
            {i18n.t('advanced.power_desc')}
          </p>
        </div>
        <span class="rounded-full border border-edge bg-canvas px-3 py-1 text-xs font-bold text-glorious">
          {sleepReadout}
        </span>
      </div>

      <div class="mt-4 flex flex-wrap gap-2">
        {#each SLEEP_OPTIONS as opt (opt.seconds)}
          {@const isSelected = draft.sleep_seconds === opt.seconds}
          <button
            type="button"
            onclick={() => setSleep(opt.seconds)}
            aria-pressed={isSelected}
            class="rounded-xl border px-3.5 py-1.5 text-xs font-bold transition {isSelected
              ? 'border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/60 shadow-[0_0_12px_rgba(255,184,0,0.18)]'
              : 'border-edge bg-canvas/60 text-ink-dim hover:border-edge-bright hover:text-ink'}"
          >
            {opt.seconds === 'off' ? i18n.t('lighting.sleep_off') : opt.label}
          </button>
        {/each}
      </div>

      <p class="mt-3 text-[10px] text-ink-faint">
        {i18n.t('advanced.power_hint')}
      </p>
    </section>

    <!-- Two-column: Left Click Lock & Scrolling Direction -->
    <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
      <!-- Left Click Lock (Safety Feature) -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Lock size={14} class="text-glorious" />
              <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
                {i18n.t('advanced.left_lock_title')}
              </h3>
            </div>
            {#if draft.left_click_lock}
              <span class="rounded-full border border-emerald-500/30 bg-emerald-950/40 px-2 py-0.5 text-[10px] font-bold text-emerald-400">
                {i18n.t('advanced.left_lock_badge')}
              </span>
            {/if}
          </div>
          <p class="mt-1 text-xs text-ink-dim">
            {i18n.t('advanced.left_lock_desc')}
          </p>
        </div>

        <div class="mt-4 flex items-center justify-between rounded-xl border border-edge bg-canvas/60 p-3">
          <div class="text-xs font-semibold text-ink">
            {draft.left_click_lock ? i18n.t('advanced.left_lock_enabled') : i18n.t('advanced.left_lock_disabled')}
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={draft.left_click_lock}
            aria-label="Toggle Left Click Lock"
            onclick={() => onChange((prev) => ({ ...prev, left_click_lock: !prev.left_click_lock }))}
            class="flex h-6 w-11 shrink-0 items-center rounded-full border px-0.5 transition cursor-pointer {draft.left_click_lock ? 'border-glorious/60 bg-glorious/25' : 'border-edge bg-edge'}"
          >
            <span class="h-[18px] w-[18px] rounded-full transition-transform duration-200 {draft.left_click_lock ? 'translate-x-[22px] bg-glorious' : 'translate-x-0 bg-ink-dim'}"></span>
          </button>
        </div>
      </section>

      <!-- Scroll Direction Toggle -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              {i18n.t('advanced.scroll_title')}
            </h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious capitalize">
              {i18n.t(`advanced.scroll_${draft.scroll_direction}`)}
            </span>
          </div>
          <p class="mt-1 text-xs text-ink-dim">
            {i18n.t('advanced.scroll_desc')}
          </p>
        </div>

        <div class="mt-4 grid grid-cols-2 gap-3">
          {#each SCROLL_MODES as mode (mode.value)}
            {@const isSelected = draft.scroll_direction === mode.value}
            <button
              type="button"
              onclick={() => setScrollDirection(mode.value)}
              aria-pressed={isSelected}
              class="flex flex-col items-start rounded-xl border p-3 text-left transition {isSelected
                ? 'border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/60 shadow-[0_0_14px_rgba(255,184,0,0.18)]'
                : 'border-edge bg-canvas/60 text-ink-dim hover:border-edge-bright hover:text-ink'}"
            >
              <span class="text-sm font-extrabold">{i18n.t(mode.labelKey)}</span>
              <span class="mt-0.5 text-[10px] text-ink-faint">{i18n.t(mode.hintKey)}</span>
            </button>
          {/each}
        </div>
      </section>
    </div>

    <!-- Conditional: Sensor DSP Pipeline (Only shown when hardware supports it) -->
    {#if hasAnySensorDsp}
      <section class="rounded-2xl border border-edge bg-card p-5">
        <div class="mb-4">
          <div class="flex items-center gap-2">
            <ShieldCheck size={16} class="text-glorious" />
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              {i18n.t('advanced.dsp_title')}
            </h3>
          </div>
          <p class="mt-1 text-xs text-ink-dim">
            {i18n.t('advanced.dsp_desc')}
          </p>
        </div>

        <div class="divide-y divide-edge/60 rounded-xl border border-edge bg-canvas/60">
          <!-- Angle Snapping -->
          {#if caps?.hasAngleSnapping}
            <div class="flex items-center justify-between p-4">
              <div>
                <span class="text-xs font-bold text-ink">{i18n.t('advanced.angle_snapping')}</span>
                <p class="text-[11px] text-ink-dim">{i18n.t('advanced.angle_snapping_desc')}</p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={draft.angle_snap}
                aria-label="Toggle Angle Snapping"
                onclick={() => onChange((prev) => ({ ...prev, angle_snap: !prev.angle_snap }))}
                class="flex h-6 w-11 shrink-0 items-center rounded-full border px-0.5 transition cursor-pointer {draft.angle_snap ? 'border-glorious/60 bg-glorious/25' : 'border-edge bg-edge'}"
              >
                <span class="h-[18px] w-[18px] rounded-full transition-transform duration-200 {draft.angle_snap ? 'translate-x-[22px] bg-glorious' : 'translate-x-0 bg-ink-dim'}"></span>
              </button>
            </div>
          {/if}

          <!-- Ripple Control -->
          {#if caps?.hasRippleControl}
            <div class="flex items-center justify-between p-4">
              <div>
                <span class="text-xs font-bold text-ink">{i18n.t('advanced.ripple_control')}</span>
                <p class="text-[11px] text-ink-dim">{i18n.t('advanced.ripple_control_desc')}</p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={draft.ripple_control}
                aria-label="Toggle Ripple Control"
                onclick={() => onChange((prev) => ({ ...prev, ripple_control: !prev.ripple_control }))}
                class="flex h-6 w-11 shrink-0 items-center rounded-full border px-0.5 transition cursor-pointer {draft.ripple_control ? 'border-glorious/60 bg-glorious/25' : 'border-edge bg-edge'}"
              >
                <span class="h-[18px] w-[18px] rounded-full transition-transform duration-200 {draft.ripple_control ? 'translate-x-[22px] bg-glorious' : 'translate-x-0 bg-ink-dim'}"></span>
              </button>
            </div>
          {/if}

          <!-- Motion Sync -->
          {#if caps?.hasMotionSync}
            <div class="flex items-center justify-between p-4">
              <div>
                <span class="text-xs font-bold text-ink">{i18n.t('advanced.motion_sync')}</span>
                <p class="text-[11px] text-ink-dim">{i18n.t('advanced.motion_sync_desc')}</p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={draft.motion_sync}
                aria-label="Toggle Motion Sync"
                onclick={() => onChange((prev) => ({ ...prev, motion_sync: !prev.motion_sync }))}
                class="flex h-6 w-11 shrink-0 items-center rounded-full border px-0.5 transition cursor-pointer {draft.motion_sync ? 'border-glorious/60 bg-glorious/25' : 'border-edge bg-edge'}"
              >
                <span class="h-[18px] w-[18px] rounded-full transition-transform duration-200 {draft.motion_sync ? 'translate-x-[22px] bg-glorious' : 'translate-x-0 bg-ink-dim'}"></span>
              </button>
            </div>
          {/if}
        </div>
      </section>
    {/if}

    <!-- Factory Reset -->
    <section class="rounded-2xl border border-[#ff5c5c]/30 bg-[#2a0f12]/40 p-5">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-start gap-3">
          <span class="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-[#ff5c5c]/40 bg-[#ff5c5c]/10 text-[#ff8a8a]">
            <TriangleAlert size={18} />
          </span>
          <div>
            <h3 class="text-sm font-bold text-ink">{i18n.t('advanced.factory_title')}</h3>
            <p class="mt-1 max-w-[500px] text-xs leading-snug text-ink-dim">
              {i18n.t('advanced.factory_desc', { count: profileCount })}
            </p>
          </div>
        </div>

        <button
          type="button"
          disabled={!device || !onFactoryReset}
          onclick={requestReset}
          class="shrink-0 rounded-full border border-[#ff5c5c]/50 bg-[#ff5c5c]/10 px-5 py-2 text-xs font-bold text-[#ff8a8a] transition hover:border-[#ff5c5c] hover:bg-[#ff5c5c]/20 disabled:cursor-not-allowed disabled:opacity-40 cursor-pointer"
        >
          {i18n.t('advanced.factory_button')}
        </button>
      </div>

      {#if !device}
        <p class="mt-3 text-[11px] text-[#ffb0b0]/80">
          {i18n.t('advanced.factory_connect_prompt')}
        </p>
      {/if}
    </section>
  </div>
</div>

<!-- Confirmation overlay -->
{#if confirmReset}
  <button
    type="button"
    aria-label="Cancel factory reset"
    class="fixed inset-0 z-40 cursor-default bg-black/70 backdrop-blur-sm"
    onclick={() => (confirmReset = false)}
  ></button>

  <div class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center p-6">
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="factory-reset-title"
      class="pointer-events-auto w-full max-w-md rounded-2xl border border-[#ff5c5c]/40 bg-panel p-6 shadow-[0_24px_80px_rgba(0,0,0,0.7)]"
    >
      <div class="flex items-start gap-3">
        <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-[#ff5c5c]/40 bg-[#ff5c5c]/10 text-[#ff8a8a]">
          <TriangleAlert size={20} />
        </span>
        <div>
          <h3 id="factory-reset-title" class="text-base font-bold text-ink">
            {i18n.t('advanced.factory_modal_title')}
          </h3>
          <p class="mt-1 text-xs leading-snug text-ink-dim">
            {i18n.t('advanced.factory_modal_desc', { device: deviceLabel, count: profileCount })}
          </p>
        </div>
      </div>

      <div class="mt-6 flex justify-end gap-2">
        <button
          type="button"
          class="rounded-full border border-edge bg-card px-4 py-2 text-xs font-semibold text-ink-dim transition hover:border-edge-bright hover:text-ink cursor-pointer"
          onclick={() => (confirmReset = false)}
        >
          {i18n.t('action.cancel')}
        </button>
        <button
          type="button"
          class="rounded-full bg-[#ff5c5c] px-5 py-2 text-xs font-bold text-[#1a0507] shadow-[0_0_18px_rgba(255,92,92,0.35)] transition hover:bg-[#ff7a7a] cursor-pointer"
          onclick={confirmFactoryReset}
        >
          {i18n.t('advanced.factory_modal_confirm')}
        </button>
      </div>
    </div>
  </div>
{/if}

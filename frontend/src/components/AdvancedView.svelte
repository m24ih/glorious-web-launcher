<script lang="ts">
  import { Lock, Power, TriangleAlert } from 'lucide-svelte';
  import type { DeviceView, ScrollDirection, SettingsDraft } from '../lib/types';
  import { SLEEP_OPTIONS, clamp, rangeFill } from '../lib/actions';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
    onFactoryReset?: () => void;
    device?: DeviceView | null;
    caps?: import('../lib/types').DeviceCapabilities;
  }

  let { draft, onChange, onFactoryReset, device = null, caps }: Props = $props();

  const LOD_OPTIONS = [
    { value: 1, label: '1 mm', hint: 'Low lift-off' },
    { value: 2, label: '2 mm', hint: 'High lift-off' },
  ] as const;

  const SCROLL_MODES: Array<{ value: ScrollDirection; label: string }> = [
    { value: 'standard', label: 'Standard' },
    { value: 'reverse', label: 'Reverse' },
  ];

  let confirmReset = $state(false);

  const isConnected = $derived(
    device?.connected === true && device?.status === 'connected',
  );
  const deviceLabel = $derived(device?.name ?? 'No device');
  const profileCount = $derived(
    device?.profiles ? Math.max(Object.keys(device.profiles).length, 1) : 3,
  );

  function formatSleep(value: number | 'off'): string {
    if (value === 'off' || value === 0) return 'Disabled';
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

{#snippet toggleRow(
  label: string,
  description: string,
  value: boolean,
  onToggle: () => void,
)}
  <div
    class="flex items-center justify-between gap-4 rounded-xl border border-edge bg-canvas/50 px-4 py-3"
  >
    <div class="min-w-0">
      <div class="flex items-center gap-2">
        <span class="text-sm font-semibold text-ink">{label}</span>
        {#if value}
          <span
            class="h-1.5 w-1.5 rounded-full bg-glorious shadow-[0_0_6px_#ffb800]"
          ></span>
        {/if}
      </div>
      <p class="mt-0.5 text-[11px] leading-snug text-ink-dim">{description}</p>
    </div>
    <button
      type="button"
      role="switch"
      aria-checked={value}
      aria-label={label}
      onclick={onToggle}
      class="flex h-6 w-11 shrink-0 items-center rounded-full border px-0.5 transition {value
        ? 'border-glorious/60 bg-glorious/25 shadow-[0_0_12px_rgba(255,184,0,0.25)]'
        : 'border-edge-bright bg-edge'}"
    >
      <span
        class="h-[18px] w-[18px] rounded-full transition-transform duration-200 {value
          ? 'translate-x-[22px] bg-glorious shadow-[0_0_8px_rgba(255,184,0,0.7)]'
          : 'translate-x-0 bg-ink-dim'}"
      ></span>
    </button>
  </div>
{/snippet}

<div class="flex-1 overflow-y-auto p-8">
  <div class="mx-auto max-w-[800px] space-y-8">
    <!-- Header -->
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div>
        <h2 class="text-xl font-bold text-ink">Advanced Settings</h2>
        <p class="text-xs text-ink-dim">
          Sensor behaviour, click debounce, and wireless power management. Changes
          are staged until you press Apply to Mouse.
        </p>
      </div>
      <div
        class="flex items-center gap-2 rounded-full border border-edge bg-card px-3 py-1.5 text-[11px] text-ink-dim"
        title="Connected device"
      >
        <span
          class="h-1.5 w-1.5 rounded-full"
          style="background: {isConnected ? '#2dff6a' : '#6b6b75'}"
        ></span>
        <span class="max-w-[180px] truncate">{deviceLabel}</span>
      </div>
    </div>

    <!-- System Features -->
    <section class="rounded-2xl border border-edge bg-card p-6">
      <div class="mb-4">
        <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">
          SYSTEM FEATURES
        </h3>
        <p class="mt-1 text-xs text-ink-dim">
          Low-level sensor and button behaviour tuned for competitive play.
        </p>
      </div>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {#if caps?.hasAngleSnapping}
          {@render toggleRow(
            'Angle Snapping',
            'Locks near-horizontal or near-vertical movement to a straight line.',
            draft.angle_snap,
            () => onChange((prev) => ({ ...prev, angle_snap: !prev.angle_snap })),
          )}
        {:else}
          <div class="rounded-xl border border-amber-500/40 bg-amber-950/40 px-4 py-3 text-[11px] leading-snug text-amber-300">
            <div class="font-bold">Angle Snapping</div>
            <div>Hardware unsupported on {device?.name ?? 'this device'} (Requires Model O 2 / BAMF 2.0)</div>
          </div>
        {/if}
        {#if caps?.hasRippleControl}
          {@render toggleRow(
            'Ripple Control',
            'Tames click and scroll spam so rapid inputs stay consistent.',
            draft.ripple_control,
            () => onChange((prev) => ({ ...prev, ripple_control: !prev.ripple_control })),
          )}
        {:else}
          <div class="rounded-xl border border-amber-500/40 bg-amber-950/40 px-4 py-3 text-[11px] leading-snug text-amber-300">
            <div class="font-bold">Ripple Control</div>
            <div>Hardware unsupported on {device?.name ?? 'this device'} (Requires Model O 2 / BAMF 2.0)</div>
          </div>
        {/if}
        {#if caps?.hasMotionSync}
          {@render toggleRow(
            'Motion Sync',
            'Aligns sensor sampling with the USB report interval for smoother sweeps.',
            draft.motion_sync,
            () => onChange((prev) => ({ ...prev, motion_sync: !prev.motion_sync })),
          )}
        {:else}
          <div class="rounded-xl border border-amber-500/40 bg-amber-950/40 px-4 py-3 text-[11px] leading-snug text-amber-300">
            <div class="font-bold">Motion Sync</div>
            <div>Hardware unsupported on {device?.name ?? 'this device'} (Requires Model O 2 / BAMF 2.0)</div>
          </div>
        {/if}
        {@render toggleRow(
          'Left Click Lock',
          'Safeguard that keeps the left click usable even if a profile binding disables it.',
          draft.left_click_lock,
          () =>
            onChange((prev) => ({
              ...prev,
              left_click_lock: !prev.left_click_lock,
            })),
        )}
      </div>

      {#if !draft.left_click_lock}
        <div
          class="mt-3 flex items-start gap-2 rounded-xl border border-[#ffb800]/40 bg-[#ffb800]/[0.08] px-4 py-3 text-[11px] leading-snug text-[#ffd257]"
        >
          <Lock size={14} class="mt-0.5 shrink-0" />
          <span>
            Left Click Lock is off. A profile can now bind away the left click,
            which leaves the mouse without a primary button — enable the lock to
            prevent that.
          </span>
        </div>
      {/if}

      <div class="mt-5 grid grid-cols-1 gap-4 sm:grid-cols-2">
        <!-- Scrolling direction -->
        <div class="rounded-xl border border-edge bg-canvas/50 p-4">
          <h4 class="text-xs font-bold tracking-[0.18em] text-ink-faint">
            SCROLLING DIRECTION
          </h4>
          <p class="mt-1 text-[11px] text-ink-dim">
            Invert the wheel axis for your desktop environment.
          </p>
          <div class="mt-3 flex rounded-full border border-edge bg-canvas p-1">
            {#each SCROLL_MODES as mode (mode.value)}
              {@const isSelected = draft.scroll_direction === mode.value}
              <button
                type="button"
                class="flex-1 rounded-full px-3 py-1.5 text-xs font-bold transition {isSelected
                  ? 'bg-glorious/15 text-glorious shadow-[inset_0_0_0_1px_rgba(255,184,0,0.55)]'
                  : 'text-ink-dim hover:text-ink'}"
                aria-pressed={isSelected}
                onclick={() => setScrollDirection(mode.value)}
              >
                {mode.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Lift-off distance -->
        <div class="rounded-xl border border-edge bg-canvas/50 p-4">
          <h4 class="text-xs font-bold tracking-[0.18em] text-ink-faint">
            LIFT-OFF DISTANCE (LOD)
          </h4>
          <p class="mt-1 text-[11px] text-ink-dim">
            Sensor tracking cut-off height above the surface.
          </p>
          <div class="mt-3 flex gap-2">
            {#each LOD_OPTIONS as opt (opt.value)}
              {@const isSelected = draft.lod === opt.value}
              <button
                type="button"
                class="flex-1 rounded-full border py-2 text-xs font-bold transition {isSelected
                  ? 'border-glorious/60 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.25)]'
                  : 'border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
                aria-pressed={isSelected}
                title={opt.hint}
                onclick={() => setLod(opt.value)}
              >
                {opt.label}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </section>

    <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
      <!-- Debounce -->
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4 flex items-start justify-between gap-4">
          <div class="min-w-0 flex-1">
            <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">
              DEBOUNCE TIME
            </h3>
            <p class="mt-1 text-xs text-ink-dim">
              Switch debounce window. Lower reacts faster, higher filters chatter.
            </p>
          </div>
          <span
            class="shrink-0 whitespace-nowrap rounded-full border border-edge bg-canvas px-3 py-1 text-xs font-bold text-glorious tabular-nums"
          >
            {draft.debounce_ms} ms
          </span>
        </div>

        <input
          type="range"
          min="0"
          max="16"
          step="1"
          value={draft.debounce_ms}
          style="--fill: {rangeFill(draft.debounce_ms, 0, 16)}"
          oninput={(e) => setDebounce(parseInt(e.currentTarget.value, 10))}
          class="w-full"
          aria-label="Debounce time in milliseconds"
        />

        <div class="mt-3 flex items-center gap-3">
          <span
            class="rounded-full border border-[#ff5c5c]/40 bg-[#ff5c5c]/10 px-2.5 py-0.5 text-[10px] font-bold text-[#ff8a8a]"
          >
            FAST · 0 ms
          </span>
          <div class="h-px flex-1 bg-edge"></div>
          <span
            class="rounded-full border border-[#2dff6a]/40 bg-[#2dff6a]/10 px-2.5 py-0.5 text-[10px] font-bold text-[#7dffa8]"
          >
            SAFE · 16 ms
          </span>
        </div>
        <p class="mt-3 text-[11px] leading-snug text-ink-faint">
          {#if draft.debounce_ms <= 3}
            Fastest response — best for tracking-heavy aiming, but chatter on worn
            switches can double-fire.
          {:else if draft.debounce_ms <= 8}
            Balanced — snappy clicks with a small safety margin.
          {:else}
            Safest — filters switch bounce aggressively at the cost of click
            latency.
          {/if}
        </p>
      </section>

      <!-- Power management -->
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4 flex items-start justify-between gap-4">
          <div class="min-w-0 flex-1">
            <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">
              POWER MANAGEMENT
            </h3>
            <p class="mt-1 text-xs text-ink-dim">
              Wireless sleep timer — idle time before the mouse drops into
              low-power mode.
            </p>
          </div>
          <span
            class="flex shrink-0 items-center gap-1.5 whitespace-nowrap rounded-full border border-edge bg-canvas px-3 py-1 text-xs font-bold text-glorious tabular-nums"
          >
            <Power size={12} />
            {sleepReadout}
          </span>
        </div>

        <div class="grid grid-cols-4 gap-2">
          {#each SLEEP_OPTIONS as opt (opt.label)}
            {@const isSelected = draft.sleep_seconds === opt.seconds}
            <button
              type="button"
              class="rounded-full border py-2 text-[11px] font-bold transition {isSelected
                ? 'border-glorious/60 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.25)]'
                : 'border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
              aria-pressed={isSelected}
              onclick={() => setSleep(opt.seconds)}
            >
              {opt.label}
            </button>
          {/each}
        </div>

        <p class="mt-4 text-[11px] leading-snug text-ink-faint">
          Current timer readout: <span class="font-semibold text-ink-dim">{sleepReadout}</span>.
          Applies to the wireless session; a wired connection stays awake.
        </p>
      </section>
    </div>

    <!-- Factory reset -->
    <section
      class="rounded-2xl border border-[#ff5c5c]/35 bg-[#2a0f12]/60 p-6"
    >
      <div
        class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div class="flex items-start gap-3">
          <span
            class="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-[#ff5c5c]/40 bg-[#ff5c5c]/10 text-[#ff8a8a]"
          >
            <TriangleAlert size={18} />
          </span>
          <div>
            <h3 class="text-sm font-bold text-ink">Factory Reset</h3>
            <p class="mt-1 max-w-[460px] text-xs leading-snug text-ink-dim">
              Restore all {profileCount} hardware profiles to factory defaults —
              DPI stages, polling rate, debounce, LOD, sleep timer, button
              bindings, and lighting. This cannot be undone.
            </p>
          </div>
        </div>

        <button
          type="button"
          disabled={!device || !onFactoryReset}
          onclick={requestReset}
          class="shrink-0 rounded-full border border-[#ff5c5c]/50 bg-[#ff5c5c]/10 px-5 py-2 text-xs font-bold text-[#ff8a8a] transition hover:border-[#ff5c5c] hover:bg-[#ff5c5c]/20 disabled:cursor-not-allowed disabled:opacity-40"
        >
          Reset Hardware
        </button>
      </div>

      {#if !device}
        <p class="mt-4 text-[11px] text-[#ffb0b0]/80">
          Connect a Glorious mouse to enable a hardware factory reset.
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
        <span
          class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-[#ff5c5c]/40 bg-[#ff5c5c]/10 text-[#ff8a8a]"
        >
          <TriangleAlert size={20} />
        </span>
        <div>
          <h3 id="factory-reset-title" class="text-base font-bold text-ink">
            Reset every hardware profile?
          </h3>
          <p class="mt-1 text-xs leading-snug text-ink-dim">
            {deviceLabel} will be rewritten with factory defaults for all
            {profileCount} profiles. Custom DPI stages, colors, lighting, and
            button bindings will be lost.
          </p>
        </div>
      </div>

      <div class="mt-6 flex justify-end gap-2">
        <button
          type="button"
          class="rounded-full border border-edge bg-card px-4 py-2 text-xs font-semibold text-ink-dim transition hover:border-edge-bright hover:text-ink"
          onclick={() => (confirmReset = false)}
        >
          Cancel
        </button>
        <button
          type="button"
          class="rounded-full bg-[#ff5c5c] px-5 py-2 text-xs font-bold text-[#1a0507] shadow-[0_0_18px_rgba(255,92,92,0.35)] transition hover:bg-[#ff7a7a]"
          onclick={confirmFactoryReset}
        >
          Yes, reset everything
        </button>
      </div>
    </div>
  </div>
{/if}

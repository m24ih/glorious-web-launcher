<script lang="ts">
  import { Lock, Power, TriangleAlert, ShieldCheck } from 'lucide-svelte';
  import type { DeviceView, ScrollDirection, SettingsDraft } from '../lib/types';
  import { SLEEP_OPTIONS, clamp } from '../lib/actions';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
    onFactoryReset?: () => void;
    device?: DeviceView | null;
    caps?: import('../lib/types').DeviceCapabilities;
  }

  let { draft, onChange, onFactoryReset, device = null, caps }: Props = $props();

  const LOD_OPTIONS = [
    { value: 1, label: '1.0 mm', hint: 'Low (Competitive tracking)' },
    { value: 2, label: '2.0 mm', hint: 'High (Thick mousepad / rough surfaces)' },
  ] as const;

  const SCROLL_MODES: Array<{ value: ScrollDirection; label: string }> = [
    { value: 'standard', label: 'Standard' },
    { value: 'reverse', label: 'Reverse' },
  ];

  let confirmReset = $state(false);

  const isConnected = $derived(
    device?.connected === true && device?.status === 'connected',
  );
  const deviceLabel = $derived(device?.name ?? 'Glorious Device');
  const profileCount = $derived(
    device?.profiles ? Math.max(Object.keys(device.profiles).length, 1) : 3,
  );

  // Check whether any advanced sensor DSP features are actually supported on this hardware
  const hasAnySensorDsp = $derived(
    Boolean(caps?.hasAngleSnapping || caps?.hasRippleControl || caps?.hasMotionSync)
  );

  function formatSleep(value: number | 'off'): string {
    if (value === 'off' || value === 0) return 'Disabled (Always On)';
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
        <h2 class="text-lg font-bold tracking-tight text-ink">Advanced & System Settings</h2>
        <p class="text-xs text-ink-dim">
          Surface tracking distance, switch debounce filtering, wireless power management, and safety locks.
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
        <span class="max-w-[200px] truncate font-medium">{deviceLabel}</span>
      </div>
    </div>

    <!-- Lift-Off Distance & Debounce Time Row -->
    <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
      <!-- Lift-Off Distance (LOD) -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-1">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              LIFT-OFF DISTANCE (LOD)
            </h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious">
              {draft.lod}.0 mm
            </span>
          </div>
          <p class="text-xs text-ink-dim mb-4">
            The maximum height at which mouse movement is still detected and tracked by the sensor.
          </p>
        </div>

        <div>
          <div class="flex gap-3">
            {#each LOD_OPTIONS as opt (opt.value)}
              {@const isSelected = draft.lod === opt.value}
              <button
                type="button"
                class="flex-1 rounded-xl p-3 text-left transition cursor-pointer {isSelected
                  ? 'border border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/50 shadow-[0_0_14px_rgba(255,184,0,0.20)]'
                  : 'border border-edge bg-canvas/60 text-ink-dim hover:border-edge-bright hover:text-ink'}"
                aria-pressed={isSelected}
                onclick={() => setLod(opt.value)}
              >
                <div class="text-sm font-extrabold">{opt.label}</div>
                <div class="text-[10px] text-ink-faint mt-1 leading-snug">{opt.hint}</div>
              </button>
            {/each}
          </div>
          <div class="mt-3 flex items-center gap-1.5 text-[10px] text-ink-faint">
            <ShieldCheck size={12} class="text-emerald-400" />
            <span>Calibrated for Glorious Model O Wireless sensor LOD registers.</span>
          </div>
        </div>
      </section>

      <!-- Debounce Time -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-1">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              DEBOUNCE TIME
            </h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious tabular-nums">
              {draft.debounce_ms} ms
            </span>
          </div>
          <p class="text-xs text-ink-dim mb-3">
            Mechanical switch debounce window to filter contact bounce and prevent double-clicks.
          </p>
        </div>

        <div>
          <div class="relative py-2">
            <input
              type="range"
              min="0"
              max="16"
              step="1"
              value={draft.debounce_ms}
              style="--fill: {(draft.debounce_ms / 16) * 100}%"
              oninput={(e) => setDebounce(parseInt(e.currentTarget.value, 10))}
              class="w-full cursor-pointer"
              aria-label="Debounce time in milliseconds"
            />
          </div>

          <div class="mt-2 flex items-center justify-between text-[10px] font-medium text-ink-faint">
            <span class="text-[#ff8a8a]">0 ms (Fastest response)</span>
            <span class="text-[#7dffa8]">16 ms (Safest filter)</span>
          </div>

          <p class="mt-2 text-[11px] leading-snug text-ink-faint">
            {#if draft.debounce_ms <= 2}
              Ultra-fast latency for esports. Switch wear may cause inadvertent double clicks.
            {:else if draft.debounce_ms <= 8}
              Optimal competitive balance — snappy clicks with switch chatter protection.
            {:else}
              Conservative debounce — maximum double-click filtering for aged switches.
            {/if}
          </p>
        </div>
      </section>
    </div>

    <!-- Power Management (Wireless Sleep Timer) -->
    <section class="rounded-2xl border border-edge bg-card p-5">
      <div class="mb-3 flex items-start justify-between gap-4">
        <div>
          <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
            POWER MANAGEMENT (STANDBY TIMER)
          </h3>
          <p class="text-xs text-ink-dim">
            Idle duration before the mouse enters low-power sleep mode over 2.4 GHz wireless.
          </p>
        </div>
        <span class="flex shrink-0 items-center gap-1.5 rounded-full border border-edge bg-canvas px-3 py-1 text-[11px] font-bold text-glorious tabular-nums">
          <Power size={12} />
          {sleepReadout}
        </span>
      </div>

      <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-4 lg:grid-cols-8">
        {#each SLEEP_OPTIONS as opt (opt.label)}
          {@const isSelected = draft.sleep_seconds === opt.seconds}
          <button
            type="button"
            class="rounded-xl border py-2.5 text-center text-xs font-bold transition cursor-pointer {isSelected
              ? 'border-glorious/80 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.20)] ring-1 ring-glorious/40'
              : 'border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
            aria-pressed={isSelected}
            onclick={() => setSleep(opt.seconds)}
          >
            {opt.label}
          </button>
        {/each}
      </div>

      <p class="mt-3 text-[11px] text-ink-faint">
        Sleep timer applies to 2.4 GHz battery sessions. When plugged in via USB-C cable, the mouse remains awake continuously.
      </p>
    </section>

    <!-- System Safety & Controls Row: Left Click Lock & Scrolling Direction -->
    <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
      <!-- Left Click Lock -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-1">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              LEFT CLICK LOCK
            </h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold {draft.left_click_lock ? 'text-emerald-400' : 'text-amber-400'}">
              {draft.left_click_lock ? 'Locked' : 'Unlocked'}
            </span>
          </div>
          <p class="text-xs text-ink-dim mb-4">
            Safeguard that keeps the physical left button permanently assigned to Left Click.
          </p>
        </div>

        <div>
          <div class="flex items-center justify-between rounded-xl border border-edge bg-canvas/50 px-4 py-3">
            <span class="text-xs font-semibold text-ink">
              {draft.left_click_lock ? 'Safety Lock Enabled' : 'Safety Lock Disabled'}
            </span>
            <button
              type="button"
              role="switch"
              aria-checked={draft.left_click_lock}
              aria-label="Toggle Left Click Lock"
              onclick={() => onChange((prev) => ({ ...prev, left_click_lock: !prev.left_click_lock }))}
              class="flex h-6 w-11 shrink-0 items-center rounded-full border px-0.5 transition cursor-pointer {draft.left_click_lock
                ? 'border-glorious/60 bg-glorious/25 shadow-[0_0_12px_rgba(255,184,0,0.25)]'
                : 'border-edge-bright bg-edge'}"
            >
              <span
                class="h-[18px] w-[18px] rounded-full transition-transform duration-200 {draft.left_click_lock
                  ? 'translate-x-[22px] bg-glorious shadow-[0_0_8px_rgba(255,184,0,0.7)]'
                  : 'translate-x-0 bg-ink-dim'}"
              ></span>
            </button>
          </div>

          {#if !draft.left_click_lock}
            <div class="mt-2.5 flex items-start gap-2 rounded-xl border border-amber-500/40 bg-amber-950/20 px-3 py-2 text-[11px] leading-snug text-amber-300">
              <Lock size={13} class="mt-0.5 shrink-0" />
              <span>Left Click Lock is disabled. You can now reassign or unbind button 1.</span>
            </div>
          {/if}
        </div>
      </section>

      <!-- Scrolling Direction -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-1">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
              SCROLLING DIRECTION
            </h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious capitalize">
              {draft.scroll_direction}
            </span>
          </div>
          <p class="text-xs text-ink-dim mb-4">
            Invert the vertical scroll wheel orientation for your operating system or personal workflow.
          </p>
        </div>

        <div class="flex gap-3">
          {#each SCROLL_MODES as mode (mode.value)}
            {@const isSelected = draft.scroll_direction === mode.value}
            <button
              type="button"
              class="flex-1 rounded-xl py-3 text-xs font-bold transition text-center cursor-pointer {isSelected
                ? 'border border-glorious/80 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.20)]'
                : 'border border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
              aria-pressed={isSelected}
              onclick={() => setScrollDirection(mode.value)}
            >
              <div class="text-sm font-extrabold">{mode.label}</div>
              <div class="text-[10px] text-ink-faint mt-0.5 font-normal">
                {mode.value === 'standard' ? 'Default Desktop' : 'Inverted Wheel'}
              </div>
            </button>
          {/each}
        </div>
      </section>
    </div>

    <!-- Conditional: Advanced Sensor DSP Filters (Only displayed if hardware actually supports it) -->
    {#if hasAnySensorDsp}
      <section class="rounded-2xl border border-edge bg-card p-5">
        <div class="mb-3">
          <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
            SENSOR DSP PERFORMANCE
          </h3>
          <p class="text-xs text-ink-dim">
            Hardware trajectory smoothing, angular locking, and report sync.
          </p>
        </div>

        <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
          {#if caps?.hasAngleSnapping}
            <div class="flex items-center justify-between rounded-xl border border-edge bg-canvas/50 p-3.5">
              <div>
                <div class="text-xs font-bold text-ink">Angle Snapping</div>
                <div class="text-[10px] text-ink-dim mt-0.5">Filters trajectory jitter into straight lines.</div>
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

          {#if caps?.hasRippleControl}
            <div class="flex items-center justify-between rounded-xl border border-edge bg-canvas/50 p-3.5">
              <div>
                <div class="text-xs font-bold text-ink">Ripple Control</div>
                <div class="text-[10px] text-ink-dim mt-0.5">Dampens high-DPI micro-jitter.</div>
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

          {#if caps?.hasMotionSync}
            <div class="flex items-center justify-between rounded-xl border border-edge bg-canvas/50 p-3.5">
              <div>
                <div class="text-xs font-bold text-ink">Motion Sync</div>
                <div class="text-[10px] text-ink-dim mt-0.5">Syncs sensor frames with USB poll intervals.</div>
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

    <!-- Factory Reset (Keychron-style tactile danger card) -->
    <section class="rounded-2xl border border-[#ff5c5c]/30 bg-[#2a0f12]/40 p-5">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-start gap-3">
          <span class="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-[#ff5c5c]/40 bg-[#ff5c5c]/10 text-[#ff8a8a]">
            <TriangleAlert size={18} />
          </span>
          <div>
            <h3 class="text-sm font-bold text-ink">Factory Hardware Reset</h3>
            <p class="mt-1 max-w-[500px] text-xs leading-snug text-ink-dim">
              Restore all {profileCount} hardware onboard profiles to factory defaults —
              DPI stages, polling rate, debounce, LOD, standby timer, button
              bindings, and RGB lighting.
            </p>
          </div>
        </div>

        <button
          type="button"
          disabled={!device || !onFactoryReset}
          onclick={requestReset}
          class="shrink-0 rounded-full border border-[#ff5c5c]/50 bg-[#ff5c5c]/10 px-5 py-2 text-xs font-bold text-[#ff8a8a] transition hover:border-[#ff5c5c] hover:bg-[#ff5c5c]/20 disabled:cursor-not-allowed disabled:opacity-40 cursor-pointer"
        >
          Reset Hardware
        </button>
      </div>

      {#if !device}
        <p class="mt-3 text-[11px] text-[#ffb0b0]/80">
          Connect your Glorious mouse to enable onboard hardware reset.
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
            Reset every hardware profile?
          </h3>
          <p class="mt-1 text-xs leading-snug text-ink-dim">
            {deviceLabel} will be rewritten with factory defaults for all
            {profileCount} profiles. Custom DPI stages, colors, lighting, and
            button bindings will be permanently reset.
          </p>
        </div>
      </div>

      <div class="mt-6 flex justify-end gap-2">
        <button
          type="button"
          class="rounded-full border border-edge bg-card px-4 py-2 text-xs font-semibold text-ink-dim transition hover:border-edge-bright hover:text-ink cursor-pointer"
          onclick={() => (confirmReset = false)}
        >
          Cancel
        </button>
        <button
          type="button"
          class="rounded-full bg-[#ff5c5c] px-5 py-2 text-xs font-bold text-[#1a0507] shadow-[0_0_18px_rgba(255,92,92,0.35)] transition hover:bg-[#ff7a7a] cursor-pointer"
          onclick={confirmFactoryReset}
        >
          Yes, reset everything
        </button>
      </div>
    </div>
  </div>
{/if}

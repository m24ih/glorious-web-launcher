<script lang="ts">
  import type { SettingsDraft } from '../lib/types';
  import { DPI_COLORS, clamp } from '../lib/actions';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
  }

  let { draft, onChange }: Props = $props();

  const POLLING_RATES = [125, 250, 500, 1000] as const;
  const LOD_OPTIONS = [
    { value: 1, label: '1 mm' },
    { value: 2, label: '2 mm' },
  ];

  function setDpiValue(index: number, val: number) {
    const clamped = clamp(Math.round(val / 100) * 100, 100, 19000);
    onChange((prev) => {
      const nextStages = [...prev.dpi_stages];
      nextStages[index] = { value: clamped };
      return { ...prev, dpi_stages: nextStages };
    });
  }

  function setActiveStage(index: number) {
    onChange((prev) => ({ ...prev, active_dpi_stage: index }));
  }

  function setPollingRate(rate: number) {
    onChange((prev) => ({ ...prev, polling_rate: rate }));
  }

  function setDebounce(ms: number) {
    onChange((prev) => ({ ...prev, debounce_ms: clamp(ms, 0, 16) }));
  }

  function setLod(lod: number) {
    onChange((prev) => ({ ...prev, lod }));
  }
</script>

<div class="flex-1 overflow-y-auto p-8">
  <div class="mx-auto max-w-[800px] space-y-8">
    <!-- Header -->
    <div>
      <h2 class="text-xl font-bold text-ink">Pointer Settings</h2>
      <p class="text-xs text-ink-dim">Configure sensor resolution, polling frequency, and surface tracking.</p>
    </div>

    <!-- DPI Stages -->
    <section class="rounded-2xl border border-edge bg-card p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">DPI STAGES</h3>
        <span class="text-xs text-ink-dim">Active Stage: {draft.active_dpi_stage + 1}</span>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {#each draft.dpi_stages.slice(0, 4) as stage, i (i)}
          {@const color = DPI_COLORS[i % DPI_COLORS.length]}
          {@const isActive = draft.active_dpi_stage === i}
          <div
            class="relative flex flex-col rounded-xl border p-4 transition {isActive
              ? 'border-glorious/80 bg-zinc-900/90 shadow-[0_0_16px_rgba(255,184,0,0.15)]'
              : 'border-edge bg-canvas/50 hover:border-edge-bright'}"
          >
            <!-- Stage Header & Radio Toggle -->
            <label class="mb-3 flex cursor-pointer items-center justify-between">
              <div class="flex items-center gap-2">
                <span
                  class="h-3 w-3 rounded-full ring-2 ring-offset-2 ring-offset-card"
                  style="background: {color}; ring-color: {color}; box-shadow: 0 0 10px {color};"
                ></span>
                <span class="text-xs font-bold {isActive ? 'text-glorious' : 'text-ink-dim'}">
                  STAGE {i + 1}
                </span>
              </div>
              <input
                type="radio"
                name="active-dpi-stage"
                checked={isActive}
                onchange={() => setActiveStage(i)}
                class="accent-glorious h-4 w-4 cursor-pointer"
              />
            </label>

            <!-- Number input and DPI label -->
            <div class="mb-3 flex items-center justify-between gap-2">
              <input
                type="number"
                min="100"
                max="19000"
                step="100"
                value={stage.value}
                oninput={(e) => {
                  const val = parseInt(e.currentTarget.value, 10);
                  if (!isNaN(val)) setDpiValue(i, val);
                }}
                class="w-24 rounded-lg border border-edge bg-card px-2 py-1 text-base font-extrabold text-ink focus:border-glorious focus:outline-none"
              />
              <span class="text-xs font-semibold text-ink-faint">DPI</span>
            </div>

            <!-- Slider -->
            <input
              type="range"
              min="100"
              max="19000"
              step="100"
              value={stage.value}
              style="--fill: {((stage.value - 100) / (19000 - 100)) * 100}%"
              oninput={(e) => setDpiValue(i, parseInt(e.currentTarget.value, 10))}
              class="w-full"
            />
          </div>
        {/each}
      </div>
    </section>

    <!-- Polling Rate -->
    <section class="rounded-2xl border border-edge bg-card p-6">
      <div class="mb-4">
        <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">POLLING RATE</h3>
        <p class="mt-1 text-xs text-ink-dim">Report rate sent to USB / 2.4 GHz wireless receiver.</p>
      </div>

      <div class="flex flex-wrap gap-2.5">
        {#each POLLING_RATES as rate (rate)}
          {@const isSelected = draft.polling_rate === rate}
          <button
            type="button"
            class="rounded-full px-5 py-2 text-xs font-bold transition {isSelected
              ? 'border border-glorious/60 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.25)]'
              : 'border border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
            onclick={() => setPollingRate(rate)}
          >
            {rate} Hz
          </button>
        {/each}
      </div>
    </section>

    <!-- Debounce Time & LOD in a two-column row -->
    <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
      <!-- Debounce Time -->
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">DEBOUNCE TIME</h3>
            <p class="mt-1 text-xs text-ink-dim">Switch debounce delay to prevent double-clicks.</p>
          </div>
          <span class="rounded-full border border-edge bg-canvas px-3 py-1 text-xs font-bold text-glorious">
            {draft.debounce_ms} ms
          </span>
        </div>

        <input
          type="range"
          min="0"
          max="16"
          step="1"
          value={draft.debounce_ms}
          style="--fill: {(draft.debounce_ms / 16) * 100}%"
          oninput={(e) => setDebounce(parseInt(e.currentTarget.value, 10))}
          class="w-full"
        />
        <div class="mt-2 flex justify-between text-[10px] text-ink-faint">
          <span>0 ms (Fastest)</span>
          <span>16 ms (Safest)</span>
        </div>
      </section>

      <!-- Lift-Off Distance (LOD) -->
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4">
          <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">LIFT-OFF DISTANCE (LOD)</h3>
          <p class="mt-1 text-xs text-ink-dim">Sensor tracking cut-off height above surface.</p>
        </div>

        <div class="flex gap-3">
          {#each LOD_OPTIONS as opt (opt.value)}
            {@const isSelected = draft.lod === opt.value}
            <button
              type="button"
              class="flex-1 rounded-full py-2 text-xs font-bold transition {isSelected
                ? 'border border-glorious/60 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.25)]'
                : 'border border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
              onclick={() => setLod(opt.value)}
            >
              {opt.label}
            </button>
          {/each}
        </div>
      </section>
    </div>
  </div>
</div>

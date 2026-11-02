<script lang="ts">
  import { Plus, Trash2 } from 'lucide-svelte';
  import type { SettingsDraft } from '../lib/types';
  import type { DeviceView } from '../lib/types';
  import { DPI_COLORS, clamp } from '../lib/actions';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
    device?: DeviceView | null;
  }

  let { draft, onChange, device = null }: Props = $props();

  const maxStages = $derived((device?.slug?.startsWith('model-o') || device?.slug?.startsWith('model-d')) ? 6 : 8);

  const POLLING_RATES = [125, 250, 500, 1000] as const;
  const LOD_OPTIONS = [
    { value: 1, label: '1 mm' },
    { value: 2, label: '2 mm' },
  ];

  function setDpiValue(index: number, val: number) {
    const clamped = clamp(Math.round(val / 100) * 100, 100, 19000);
    onChange((prev) => {
      const nextStages = [...prev.dpi_stages];
      nextStages[index] = { ...nextStages[index], value: clamped };
      return { ...prev, dpi_stages: nextStages };
    });
  }

  function setStageColor(index: number, color: string) {
    onChange((prev) => {
      const nextStages = [...prev.dpi_stages];
      nextStages[index] = { ...nextStages[index], color };
      return { ...prev, dpi_stages: nextStages };
    });
  }

  function addStage() {
    if (draft.dpi_stages.length >= maxStages) return;
    onChange((prev) => {
      const nextIdx = prev.dpi_stages.length;
      const lastStage = prev.dpi_stages[nextIdx - 1];
      const nextVal = lastStage ? lastStage.value : 1600;
      const nextColor = DPI_COLORS[nextIdx % DPI_COLORS.length];
      const nextStages = [...prev.dpi_stages, { value: nextVal, color: nextColor }];
      return { ...prev, dpi_stages: nextStages };
    });
  }

  function removeStage(index: number) {
    if (draft.dpi_stages.length <= 1) return;
    onChange((prev) => {
      const stageNum = index + 1;
      const nextStages = prev.dpi_stages.filter((_, i) => i !== index);
      let nextActive = prev.active_dpi_stage;
      if (nextActive === stageNum) {
        nextActive = Math.min(stageNum, nextStages.length);
      } else if (nextActive > stageNum) {
        nextActive -= 1;
      }
      nextActive = Math.max(1, Math.min(nextActive, nextStages.length));
      return {
        ...prev,
        dpi_stages: nextStages,
        active_dpi_stage: nextActive,
      };
    });
  }

  function setActiveStage(stageNumber: number) {
    onChange((prev) => ({ ...prev, active_dpi_stage: stageNumber }));
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
        <div>
          <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">DPI STAGES</h3>
          <span class="text-xs text-ink-dim">Active Stage: {draft.active_dpi_stage} (of {draft.dpi_stages.length})</span>
        </div>
        {#if draft.dpi_stages.length < maxStages}
          <button
            type="button"
            onclick={addStage}
            class="flex items-center gap-1.5 rounded-full border border-glorious/40 bg-zinc-900 px-3.5 py-1.5 text-xs font-bold text-glorious transition hover:border-glorious hover:bg-glorious/10 shadow-[0_0_12px_rgba(255,184,0,0.15)]"
          >
            <Plus size={14} />
            <span>Add Stage</span>
          </button>
        {:else}
          <span class="rounded-full border border-edge bg-zinc-900 px-3 py-1 text-[11px] font-semibold text-ink-dim">Max {maxStages} Stages</span>
        {/if}
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {#each draft.dpi_stages as stage, i (i)}
          {@const stageNumber = i + 1}
          {@const color = stage.color || DPI_COLORS[i % DPI_COLORS.length]}
          {@const isActive = draft.active_dpi_stage === stageNumber}
          <div
            role="button"
            tabindex="0"
            onclick={() => setActiveStage(stageNumber)}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                setActiveStage(stageNumber);
              }
            }}
            class="relative flex flex-col rounded-xl border p-4 transition cursor-pointer select-none {isActive
              ? 'border-glorious bg-zinc-900/90 ring-1 ring-glorious/60 shadow-[0_0_20px_rgba(255,184,0,0.22)]'
              : 'border-edge bg-canvas/50 hover:border-edge-bright hover:bg-card-hover'}"
          >
            <!-- Stage Header & Radio Toggle -->
            <div class="mb-3 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <!-- Color swatch with native picker -->
                <label
                  class="relative flex h-5 w-5 items-center justify-center cursor-pointer rounded-full transition hover:scale-110"
                  title="Change Stage Color"
                >
                  <span
                    class="h-3.5 w-3.5 rounded-full ring-2 ring-offset-2 ring-offset-card"
                    style="background: {color}; ring-color: {color}; box-shadow: 0 0 10px {color};"
                  ></span>
                  <input
                    type="color"
                    value={color}
                    onclick={(e) => e.stopPropagation()}
                    oninput={(e) => setStageColor(i, e.currentTarget.value)}
                    class="absolute inset-0 h-full w-full opacity-0 cursor-pointer"
                  />
                </label>
                <span class="text-xs font-bold {isActive ? 'text-glorious' : 'text-ink-dim'}">
                  STAGE {stageNumber}
                </span>
                {#if isActive}
                  <span class="rounded-full bg-glorious/20 px-2 py-0.5 text-[9px] font-black uppercase tracking-wider text-glorious border border-glorious/40 shadow-[0_0_8px_rgba(255,184,0,0.3)]">
                    ACTIVE
                  </span>
                {/if}
              </div>

              <div class="flex items-center gap-2">
                {#if draft.dpi_stages.length > 1}
                  <button
                    type="button"
                    title="Delete Stage"
                    aria-label="Delete Stage"
                    onclick={(e) => {
                      e.stopPropagation();
                      removeStage(i);
                    }}
                    class="rounded p-1 text-ink-faint transition hover:bg-zinc-800 hover:text-red-400"
                  >
                    <Trash2 size={13} />
                  </button>
                {/if}

                <input
                  type="radio"
                  name="active-dpi-stage"
                  checked={isActive}
                  onchange={() => setActiveStage(stageNumber)}
                  onclick={(e) => e.stopPropagation()}
                  class="accent-glorious h-4 w-4 cursor-pointer"
                />
              </div>
            </div>

            <!-- Number input and DPI label -->
            <div class="mb-3 flex items-center justify-between gap-2">
              <input
                type="number"
                min="100"
                max="19000"
                step="100"
                value={stage.value}
                onclick={(e) => e.stopPropagation()}
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
              onclick={(e) => e.stopPropagation()}
              oninput={(e) => setDpiValue(i, parseInt(e.currentTarget.value, 10))}
              class="w-full"
            />
          </div>
        {/each}

        {#if draft.dpi_stages.length < maxStages}
          <button
            type="button"
            onclick={addStage}
            class="flex min-h-[140px] flex-col items-center justify-center gap-2 rounded-xl border border-dashed border-edge-bright/60 bg-canvas/30 p-4 text-ink-dim transition hover:border-glorious hover:bg-glorious/5 hover:text-glorious"
          >
            <div class="flex h-8 w-8 items-center justify-center rounded-full border border-edge bg-card">
              <Plus size={16} />
            </div>
            <span class="text-xs font-bold">Add Stage ({draft.dpi_stages.length}/{maxStages})</span>
          </button>
        {/if}
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

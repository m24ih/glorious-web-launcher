<script lang="ts">
  import { Plus, Trash2 } from 'lucide-svelte';
  import type { SettingsDraft } from '../lib/types';
  import type { DeviceCapabilities, DeviceView } from '../lib/types';
  import { DPI_COLORS, clamp } from '../lib/actions';
  import { i18n } from '../lib/i18n.svelte';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
    device?: DeviceView | null;
    caps?: DeviceCapabilities;
  }

  let { draft, onChange, device = null, caps }: Props = $props();

  const stageLimit = $derived(Math.min(caps?.maxDpiStages ?? 4, 8));
  const minDpi = $derived(caps?.minDpi ?? 100);
  const maxDpi = $derived(caps?.maxDpi ?? 19000);
  const pollingRates = $derived(caps?.pollingRates ?? [125, 250, 500, 1000]);
  const lodRaw = $derived(caps?.supportedLods ?? [1, 2]);
  const lodOptions = $derived(lodRaw.map((v) => ({ value: v, label: `${v} mm` })));

  // Current active stage object safely indexed
  const activeStageIndex = $derived(
    Math.max(0, Math.min(draft.active_dpi_stage - 1, draft.dpi_stages.length - 1))
  );
  const activeStage = $derived(draft.dpi_stages[activeStageIndex]);

  // DPI tick scale markers for Keychron-style ruler
  const dpiTicks = $derived([100, 400, 800, 1600, 3200, 6400, maxDpi]);

  function setDpiValue(index: number, val: number) {
    const clamped = clamp(Math.round(val / 100) * 100, minDpi, maxDpi);
    onChange((prev) => {
      const nextStages = [...prev.dpi_stages];
      nextStages[index] = { ...nextStages[index], value: clamped };
      return { ...prev, dpi_stages: nextStages };
    });
  }

  function adjustActiveDpi(delta: number) {
    if (!activeStage) return;
    setDpiValue(activeStageIndex, activeStage.value + delta);
  }

  function setStageColor(index: number, color: string) {
    onChange((prev) => {
      const nextStages = [...prev.dpi_stages];
      nextStages[index] = { ...nextStages[index], color };
      return { ...prev, dpi_stages: nextStages };
    });
  }

  function addStage() {
    if (draft.dpi_stages.length >= stageLimit) return;
    onChange((prev) => {
      const nextIdx = prev.dpi_stages.length;
      const lastStage = prev.dpi_stages[nextIdx - 1];
      const nextVal = lastStage ? Math.min(lastStage.value + 400, maxDpi) : 1600;
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

  function setXySplit(val: boolean) {
    onChange((prev) => ({ ...prev, xy_split: val }));
  }

  function setCompetitiveMode(val: boolean) {
    onChange((prev) => ({ ...prev, competitive_mode: val }));
  }
</script>

<div class="flex-1 overflow-y-auto p-6 lg:p-8">
  <div class="mx-auto max-w-[860px] space-y-6">
    <!-- Header -->
    <div class="flex items-end justify-between border-b border-edge/60 pb-4">
      <div>
        <h2 class="text-lg font-bold tracking-tight text-ink">{i18n.t('pointer.title')}</h2>
        <p class="text-xs text-ink-dim">{i18n.t('pointer.subtitle')}</p>
      </div>
      <div class="flex items-center gap-2">
        <span class="rounded-full border border-edge bg-canvas px-3 py-1 text-[11px] font-semibold text-ink-dim">
          {i18n.t('pointer.levels')} <strong class="text-ink font-bold">{draft.dpi_stages.length}</strong> / {stageLimit}
        </span>
        {#if draft.dpi_stages.length < stageLimit}
          <button
            type="button"
            onclick={addStage}
            class="flex items-center gap-1.5 rounded-full border border-glorious/40 bg-zinc-900 px-3 py-1 text-[11px] font-bold text-glorious transition hover:border-glorious hover:bg-glorious/10 shadow-[0_0_10px_rgba(255,184,0,0.15)]"
          >
            <Plus size={13} />
            <span>{i18n.t('pointer.add_level')}</span>
          </button>
        {/if}
      </div>
    </div>

    <!-- Horizontal DPI Stages Strip -->
    <section class="rounded-2xl border border-edge bg-card p-5">
      <div class="mb-3 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">{i18n.t('pointer.dpi_stages')}</span>
          <span class="text-xs text-ink-dim">{i18n.t('pointer.select_card')}</span>
        </div>
        {#if caps?.hasXySplitDpi}
          <button
            type="button"
            onclick={() => setXySplit(!draft.xy_split)}
            class="flex items-center gap-1.5 rounded-full border px-3 py-1 text-[11px] font-bold transition {draft.xy_split ? 'border-glorious/60 bg-zinc-900 text-glorious shadow-[0_0_10px_rgba(255,184,0,0.15)]' : 'border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
            aria-pressed={draft.xy_split}
            title="X/Y Split DPI"
          >
            <span>{i18n.t('pointer.xy_split')}</span>
          </button>
        {/if}
      </div>

      <!-- Horizontal Stage Cards -->
      <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
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
            class="group relative flex flex-col justify-between rounded-xl border p-3.5 transition cursor-pointer select-none {isActive
              ? 'border-glorious bg-zinc-900/95 ring-1 ring-glorious/60 shadow-[0_0_18px_rgba(255,184,0,0.20)]'
              : 'border-edge bg-canvas/60 hover:border-edge-bright hover:bg-card-hover'}"
          >
            <!-- Card Header: LED swatch + Stage name + Delete button -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <!-- Color swatch with native picker -->
                <label
                  class="relative flex h-4 w-4 items-center justify-center cursor-pointer rounded-sm transition hover:scale-110"
                  title="Click to change hardware LED color"
                >
                  <span
                    class="h-2.5 w-2.5 rounded-sm ring-1 ring-white/20 shadow-sm"
                    style="background: {color}; box-shadow: 0 0 8px {color};"
                  ></span>
                  <input
                    type="color"
                    value={color}
                    onclick={(e) => e.stopPropagation()}
                    oninput={(e) => setStageColor(i, e.currentTarget.value)}
                    class="absolute inset-0 h-full w-full opacity-0 cursor-pointer"
                  />
                </label>
                <span class="text-[11px] font-bold tracking-wider {isActive ? 'text-glorious' : 'text-ink-dim'}">
                  {i18n.t('pointer.stage_n', { n: stageNumber })}
                </span>
              </div>

              {#if draft.dpi_stages.length > 1}
                <button
                  type="button"
                  title="Remove Stage"
                  aria-label="Remove Stage"
                  onclick={(e) => {
                    e.stopPropagation();
                    removeStage(i);
                  }}
                  class="rounded p-1 text-ink-faint opacity-0 group-hover:opacity-100 transition hover:bg-zinc-800 hover:text-red-400"
                >
                  <Trash2 size={12} />
                </button>
              {/if}
            </div>

            <!-- Value Display -->
            <div class="my-3">
              {#if draft.xy_split && caps?.hasXySplitDpi}
                <div class="text-xs font-bold text-ink space-y-0.5">
                  <div class="flex justify-between text-ink-dim text-[11px]">
                    <span>X:</span> <strong class="text-ink">{stage.x ?? stage.value}</strong>
                  </div>
                  <div class="flex justify-between text-ink-dim text-[11px]">
                    <span>Y:</span> <strong class="text-ink">{stage.y ?? stage.value}</strong>
                  </div>
                </div>
              {:else}
                <div class="text-2xl font-black tracking-tight {isActive ? 'text-ink font-extrabold' : 'text-ink-dim'}">
                  {stage.value}
                </div>
              {/if}
              <span class="text-[10px] font-semibold tracking-widest uppercase text-ink-faint">DPI</span>
            </div>

            <!-- Active Pill / Indicator -->
            <div class="flex items-center justify-between pt-1 border-t border-white/[0.04]">
              {#if isActive}
                <span class="flex items-center gap-1 text-[10px] font-black uppercase tracking-wider text-glorious">
                  <span class="h-1.5 w-1.5 rounded-full bg-glorious animate-pulse shadow-[0_0_6px_#ffb800]"></span>
                  {i18n.t('pointer.active')}
                </span>
              {:else}
                <span class="text-[10px] font-medium text-ink-faint group-hover:text-ink-dim">
                  {i18n.t('pointer.click_to_select')}
                </span>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      <!-- Tactile DPI Slider for Active Stage -->
      {#if activeStage}
        <div class="mt-6 rounded-xl border border-edge/80 bg-panel/70 p-4">
          <div class="mb-3 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-ink">{i18n.t('pointer.sensitivity', { n: draft.active_dpi_stage })}</span>
              <span class="text-sm font-extrabold text-glorious">{activeStage.value} DPI</span>
            </div>
            <div class="flex items-center gap-1.5">
              <button
                type="button"
                onclick={() => adjustActiveDpi(-100)}
                class="rounded-md border border-edge bg-canvas px-2 py-1 text-[11px] font-bold text-ink-dim hover:border-edge-bright hover:text-ink transition"
                title="-100 DPI"
              >
                -100
              </button>
              <input
                type="number"
                min={minDpi}
                max={maxDpi}
                step="100"
                value={activeStage.value}
                oninput={(e) => {
                  const val = parseInt(e.currentTarget.value, 10);
                  if (!isNaN(val)) setDpiValue(activeStageIndex, val);
                }}
                class="w-20 rounded-md border border-edge bg-card px-2 py-1 text-center text-xs font-extrabold text-ink focus:border-glorious focus:outline-none"
              />
              <button
                type="button"
                onclick={() => adjustActiveDpi(100)}
                class="rounded-md border border-edge bg-canvas px-2 py-1 text-[11px] font-bold text-ink-dim hover:border-edge-bright hover:text-ink transition"
                title="+100 DPI"
              >
                +100
              </button>
            </div>
          </div>

          <!-- Tactile Slider Track with 3D Thumb -->
          <div class="relative py-2">
            <input
              type="range"
              min={minDpi}
              max={maxDpi}
              step="100"
              value={activeStage.value}
              style="--fill: {((activeStage.value - minDpi) / (maxDpi - minDpi)) * 100}%"
              oninput={(e) => setDpiValue(activeStageIndex, parseInt(e.currentTarget.value, 10))}
              class="w-full cursor-pointer"
            />
          </div>

          <!-- Ruler Scale Ticks -->
          <div class="mt-2 flex justify-between text-[10px] font-medium text-ink-faint">
            {#each dpiTicks as tick (tick)}
              {@const isNearest = Math.abs(activeStage.value - tick) < 200}
              <button
                type="button"
                onclick={() => setDpiValue(activeStageIndex, tick)}
                class="hover:text-ink transition {isNearest ? 'text-glorious font-bold' : ''}"
              >
                {tick}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </section>

    <!-- Polling Rate -->
    <section class="rounded-2xl border border-edge bg-card p-5">
      <div class="mb-3 flex items-center justify-between">
        <div>
          <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">{i18n.t('pointer.polling_rate')}</h3>
          <p class="text-xs text-ink-dim">{i18n.t('pointer.polling_rate_desc')}</p>
        </div>
        <span class="rounded-full border border-edge bg-canvas px-3 py-1 text-[11px] font-bold text-glorious">
          {i18n.t('pointer.polling_active', { rate: draft.polling_rate })}
        </span>
      </div>

      <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
        {#each pollingRates as rate (rate)}
          {@const isSelected = draft.polling_rate === rate}
          {@const isGaming = rate >= 1000}
          <button
            type="button"
            class="flex flex-col items-center justify-center rounded-xl p-3 text-center transition cursor-pointer {isSelected
              ? 'border border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/50 shadow-[0_0_14px_rgba(255,184,0,0.22)]'
              : 'border border-edge bg-canvas/50 text-ink-dim hover:border-edge-bright hover:text-ink'}"
            onclick={() => setPollingRate(rate)}
          >
            <div class="text-base font-black tracking-tight">{rate} Hz</div>
            <div class="mt-1 text-[10px] font-semibold uppercase tracking-wider {isGaming ? (isSelected ? 'text-glorious' : 'text-emerald-400') : 'text-ink-faint'}">
              {isGaming ? i18n.t('pointer.esports_gaming') : i18n.t('pointer.office_casual')}
            </div>
          </button>
        {/each}
      </div>
    </section>

    <!-- Debounce Time & Lift-Off Distance (Two-column cards) -->
    <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
      <!-- Lift-Off Distance (LOD) -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-1">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">{i18n.t('pointer.lod')}</h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious">
              {draft.lod} mm
            </span>
          </div>
          <p class="text-xs text-ink-dim mb-4">{i18n.t('pointer.lod_desc')}</p>
        </div>

        <div class="flex gap-3">
          {#each lodOptions as opt (opt.value)}
            {@const isSelected = draft.lod === opt.value}
            <button
              type="button"
              class="flex-1 rounded-xl py-2.5 text-xs font-bold transition text-center {isSelected
                ? 'border border-glorious/80 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.20)]'
                : 'border border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
              onclick={() => setLod(opt.value)}
            >
              <div class="text-sm font-extrabold">{opt.label}</div>
              <div class="text-[10px] text-ink-faint mt-0.5 font-normal">
                {opt.value === 1 ? i18n.t('pointer.lod_low') : i18n.t('pointer.lod_high')}
              </div>
            </button>
          {/each}
        </div>
      </section>

      <!-- Debounce Time -->
      <section class="rounded-2xl border border-edge bg-card p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-1">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">{i18n.t('pointer.debounce')}</h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious">
              {draft.debounce_ms} ms
            </span>
          </div>
          <p class="text-xs text-ink-dim mb-3">{i18n.t('pointer.debounce_desc')}</p>
        </div>

        <div>
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
          <div class="mt-2 flex justify-between text-[10px] text-ink-faint font-medium">
            <span>{i18n.t('pointer.debounce_fastest')}</span>
            <span>{i18n.t('pointer.debounce_safest')}</span>
          </div>
        </div>
      </section>
    </div>

    <!-- Conditional: Competitive Mode -->
    {#if caps?.hasCompetitiveMode}
      <section class="rounded-2xl border border-edge bg-card p-5">
        <div class="mb-3">
          <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">{i18n.t('pointer.tracking_mode')}</h3>
          <p class="text-xs text-ink-dim">{i18n.t('pointer.tracking_mode_desc')}</p>
        </div>

        <div class="flex gap-3">
          <button
            type="button"
            onclick={() => setCompetitiveMode(false)}
            class="flex-1 rounded-xl border p-3.5 text-left transition {!draft.competitive_mode ? 'border-glorious/70 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.20)]' : 'border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
            aria-pressed={!draft.competitive_mode}
          >
            <div class="text-xs font-black tracking-wide">{i18n.t('pointer.standard_mode')}</div>
            <div class="text-[11px] text-ink-dim mt-0.5">{i18n.t('pointer.standard_mode_desc')}</div>
          </button>
          <button
            type="button"
            onclick={() => setCompetitiveMode(true)}
            class="flex-1 rounded-xl border p-3.5 text-left transition {draft.competitive_mode ? 'border-glorious/70 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.20)]' : 'border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
            aria-pressed={draft.competitive_mode}
          >
            <div class="text-xs font-black tracking-wide">{i18n.t('pointer.competitive_mode')}</div>
            <div class="text-[11px] text-ink-dim mt-0.5">{i18n.t('pointer.competitive_mode_desc')}</div>
          </button>
        </div>
      </section>
    {/if}
  </div>
</div>

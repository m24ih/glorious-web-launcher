<script lang="ts">
  import { RefreshCw, Send } from 'lucide-svelte';
  import { PIN_SPECS, DPI_COLORS, PIN_LABELS } from '../lib/actions';
  import type { DeviceView, PinAction, PinId } from '../lib/types';

  interface Props {
    device: DeviceView | null;
    mappings: PinAction[];
    selected: PinId | null;
    canApply: boolean;
    applying: boolean;
    onSelect: (pin: PinId) => void;
    onReset: () => void;
    onApply: () => void;
  }

  let {
    device,
    mappings,
    selected,
    canApply,
    applying,
    onSelect,
    onReset,
    onApply,
  }: Props = $props();

  const mouseSrc = $derived(
    device ? `/mice/${device.slug}.webp` : '/mice/fallback.webp',
  );

  /** Elbow/curve control point: horizontal run from the pill towards the mouse. */
  function ctrlX(spec: (typeof PIN_SPECS)[number]): number {
    const gap = spec.side === 'left' ? 8 : -8;
    return spec.side === 'left'
      ? Math.min(spec.anchorX, spec.pillX + gap)
      : Math.max(spec.anchorX, spec.pillX + gap);
  }

  function mappingFor(pin: PinId): PinAction | undefined {
    return mappings.find((m) => m.pin === pin);
  }

  function stageColor(index: number): string {
    return DPI_COLORS[index % DPI_COLORS.length];
  }
</script>

<div class="flex min-h-0 flex-1 flex-col gap-3">
  <!-- Header -->
  <div class="flex items-center gap-3">
    <div class="min-w-0">
      <h1 class="truncate text-base font-semibold text-ink" translate="no">
        {device?.name ?? 'No device'}
      </h1>
    </div>

    {#if device?.battery}
      <div class="flex items-center gap-2 rounded border border-edge bg-card px-2.5 py-1">
        <span class="text-[11px] font-semibold text-ink">{Math.max(device.battery.percentage, 1)}%</span>
        <div class="h-1.5 w-12 overflow-hidden rounded-full bg-edge">
          <div
            class="h-full rounded-full bg-glorious"
            style="width: {Math.max(device.battery.percentage, 1)}%"
          ></div>
        </div>
      </div>
    {/if}

    {#if device?.connected}
      <div class="rounded border border-edge bg-card px-2.5 py-1 text-[11px] text-ink-dim">
        {device.connection?.type ?? 'Connected'}
      </div>
    {/if}

    <div class="ml-auto flex items-center gap-2">
      <button
        type="button"
        class="flex items-center gap-2 rounded border border-edge bg-card px-3 py-1.5 text-xs font-semibold text-ink-dim hover:border-edge-bright hover:text-ink"
        onclick={onReset}
      >
        <RefreshCw size={14} />
        Reset
      </button>
      <button
        type="button"
        class="flex items-center gap-2 rounded bg-glorious px-4 py-1.5 text-xs font-bold text-canvas hover:bg-glorious-soft disabled:cursor-not-allowed disabled:opacity-40"
        disabled={!canApply || applying}
        onclick={onApply}
      >
        <Send size={14} />
        {applying ? 'Applying…' : 'Apply to Mouse'}
      </button>
    </div>
  </div>

  <!-- Stage canvas -->
  <div class="relative flex min-h-0 flex-1 items-center justify-center">
    <div class="relative h-full max-h-[600px] w-full max-w-[1120px]">
      <!-- aspect locked canvas so %-geometry stays calibrated -->
      <div class="relative mx-auto aspect-[8/5] h-full max-h-full">
        <!-- callout lines -->
        <svg
          class="absolute inset-0 h-full w-full"
          viewBox="0 0 100 100"
          preserveAspectRatio="none"
          aria-hidden="true"
        >
          {#each PIN_SPECS as spec (spec.id)}
            {@const active = selected === spec.id}
            <path
              d="M {spec.pillX} {spec.pillY} Q {ctrlX(spec)} {spec.pillY} {spec.anchorX} {spec.anchorY}"
              fill="none"
              vector-effect="non-scaling-stroke"
              stroke={active ? '#6277ff' : '#34343b'}
              stroke-width={active ? 1.6 : 1.1}
              stroke-linecap="round"
              class="transition-all"
            />
            <circle
              cx={spec.anchorX}
              cy={spec.anchorY}
              r={active ? 0.55 : 0.35}
              vector-effect="non-scaling-stroke"
              fill={active ? '#6277ff' : '#4a4a52'}
              class="transition-all"
            />
          {/each}
        </svg>

        <!-- mouse render -->
        <img
          src={mouseSrc}
          alt={device ? `${device.name} render` : 'Mouse render'}
          class="pointer-events-none absolute left-1/2 top-1/2 h-[88%] max-w-none -translate-x-1/2 -translate-y-1/2 select-none object-contain"
          draggable="false"
          onerror={(e) => {
            const img = e.currentTarget as HTMLImageElement;
            if (!img.src.endsWith('/mice/fallback.webp')) img.src = '/mice/fallback.webp';
          }}
        />

        <!-- callout pills -->
        {#each PIN_SPECS as spec, i (spec.id)}
          {@const active = selected === spec.id}
          {@const mapped = mappingFor(spec.id)}
          <button
            type="button"
            class="absolute flex flex-col items-center rounded border px-2.5 py-1 text-left {active
              ? 'border-[#6277ff] bg-[#4657c9] shadow-sm'
              : 'border-edge bg-card hover:border-edge-bright'}"
            style={spec.side === 'left'
              ? `left: ${spec.pillX}%; top: ${spec.pillY}%; transform: translate(-50%, -50%)`
              : `left: ${spec.pillX}%; top: ${spec.pillY}%; transform: translate(-50%, -50%)`}
            onclick={() => onSelect(spec.id)}
            title="Select to remap {PIN_LABELS[spec.id]}"
          >
            <span
              class="text-[10px] font-semibold leading-tight {active ? 'text-white' : 'text-ink'}"
            >
              {spec.label}
            </span>
            {#if mapped && mapped.action !== PIN_LABELS[spec.id]}
              <span class="mt-0.5 rounded-full px-1.5 text-[9px] leading-tight" style="background: {stageColor(i)}22; color: {stageColor(i)}">
                {mapped.action}
              </span>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<script lang="ts">
  import { onMount } from 'svelte';
  import { Activity, Crosshair, Eraser, Info, RotateCcw } from 'lucide-svelte';
  import type { DeviceView, SettingsDraft } from '../lib/types';
  import {
    SENSOR_ANGLE_MAX,
    SENSOR_ANGLE_MIN,
    clampSensorAngle,
    rangeFill,
  } from '../lib/actions';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
    device?: DeviceView | null;
  }

  let { draft, onChange, device = null }: Props = $props();

  /** Keychron standard quick preset angles */
  const ANGLE_PRESETS = [-10, -5, -1, 0, 1, 5, 10] as const;
  const ANGLE_TICKS = [-10, -5, 0, 5, 10] as const;

  /** Upper bound on stored trajectory points; older strokes scroll off. */
  const MAX_POINTS = 6000;

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let ctx: CanvasRenderingContext2D | null = null;

  let strokes: Array<Array<{ x: number; y: number }>> = [];
  let drawing = false;
  let openStroke = false;

  let hasInk = $state(false);
  let sampleCount = $state(0);

  const isConnected = $derived(
    device?.connected === true && device?.status === 'connected',
  );
  const deviceLabel = $derived(device?.name ?? 'No device');
  const isNeutral = $derived(draft.sensor_angle === 0);

  function formatAngle(value: number): string {
    const rounded = Math.round(value * 10) / 10;
    if (rounded > 0) return `+${rounded.toFixed(1)}°`;
    if (rounded < 0) return `${rounded.toFixed(1)}°`;
    return '0.0°';
  }

  function setAngle(value: number) {
    onChange((prev) => ({
      ...prev,
      sensor_angle: clampSensorAngle(value),
    }));
  }

  /* ── Canvas sizing ───────────────────────────────────────────────────── */

  function sizeCanvas() {
    if (!canvasEl) return;
    const width = canvasEl.clientWidth;
    const height = canvasEl.clientHeight;
    if (width <= 0 || height <= 0) return;

    const dpr = window.devicePixelRatio || 1;
    canvasEl.width = Math.max(1, Math.round(width * dpr));
    canvasEl.height = Math.max(1, Math.round(height * dpr));

    ctx = canvasEl.getContext('2d');
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    redraw(draft.sensor_angle);
  }

  /* ── Rendering ───────────────────────────────────────────────────────── */

  function redraw(angle: number) {
    if (!ctx || !canvasEl) return;
    const width = canvasEl.clientWidth;
    const height = canvasEl.clientHeight;
    if (width <= 0 || height <= 0) return;

    ctx.clearRect(0, 0, width, height);

    // Background grid.
    ctx.save();
    ctx.strokeStyle = 'rgba(58, 58, 68, 0.55)';
    ctx.lineWidth = 1;
    const step = 32;
    for (let x = step; x < width; x += step) {
      ctx.beginPath();
      ctx.moveTo(x + 0.5, 0);
      ctx.lineTo(x + 0.5, height);
      ctx.stroke();
    }
    for (let y = step; y < height; y += step) {
      ctx.beginPath();
      ctx.moveTo(0, y + 0.5);
      ctx.lineTo(width, y + 0.5);
      ctx.stroke();
    }

    // Raw horizontal reference.
    ctx.strokeStyle = 'rgba(107, 107, 117, 0.75)';
    ctx.setLineDash([6, 6]);
    ctx.beginPath();
    ctx.moveTo(0, height / 2);
    ctx.lineTo(width, height / 2);
    ctx.stroke();

    // Corrected axis: rotated by the configured sensor angle.
    const radians = (angle * Math.PI) / 180;
    const cx = width / 2;
    const cy = height / 2;
    const reach = Math.hypot(width, height);
    ctx.setLineDash([]);
    ctx.strokeStyle = 'rgba(255, 184, 0, 0.55)';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(cx - Math.cos(radians) * reach, cy - Math.sin(radians) * reach);
    ctx.lineTo(cx + Math.cos(radians) * reach, cy + Math.sin(radians) * reach);
    ctx.stroke();

    // Center marker.
    ctx.fillStyle = 'rgba(255, 184, 0, 0.5)';
    ctx.beginPath();
    ctx.arc(cx, cy, 2, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();

    // Trajectory strokes.
    ctx.save();
    ctx.strokeStyle = '#ffb800';
    ctx.fillStyle = '#ffb800';
    ctx.lineWidth = 2;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.shadowColor = 'rgba(255, 184, 0, 0.65)';
    ctx.shadowBlur = 8;

    for (const stroke of strokes) {
      if (stroke.length === 0) continue;
      if (stroke.length === 1) {
        ctx.beginPath();
        ctx.arc(stroke[0].x, stroke[0].y, 1.6, 0, Math.PI * 2);
        ctx.fill();
        continue;
      }
      ctx.beginPath();
      ctx.moveTo(stroke[0].x, stroke[0].y);
      for (let i = 1; i < stroke.length; i += 1) {
        ctx.lineTo(stroke[i].x, stroke[i].y);
      }
      ctx.stroke();
    }
    ctx.restore();
  }

  /* ── Pointer drawing ─────────────────────────────────────────────────── */

  function pointerPosition(event: PointerEvent) {
    if (!canvasEl) return { x: 0, y: 0 };
    const rect = canvasEl.getBoundingClientRect();
    return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  }

  function trimStrokes() {
    let total = strokes.reduce((sum, stroke) => sum + stroke.length, 0);
    while (total > MAX_POINTS && strokes.length > 1) {
      total -= strokes[0].length;
      strokes.shift();
    }
  }

  function appendPoint(point: { x: number; y: number }) {
    if (!openStroke) {
      strokes.push([point]);
      openStroke = true;
      sampleCount += 1;
    } else {
      const current = strokes[strokes.length - 1];
      const last = current[current.length - 1];
      if (last && Math.hypot(point.x - last.x, point.y - last.y) < 1.5) return;
      current.push(point);
      sampleCount += 1;
      trimStrokes();
    }
    hasInk = true;
    redraw(draft.sensor_angle);
  }

  function onPointerEnter() {
    drawing = true;
    openStroke = false;
  }

  function onPointerDown(event: PointerEvent) {
    drawing = true;
    openStroke = false;
    appendPoint(pointerPosition(event));
  }

  function onPointerMove(event: PointerEvent) {
    if (!drawing) return;
    appendPoint(pointerPosition(event));
  }

  function onPointerLeave() {
    drawing = false;
    openStroke = false;
  }

  function clearCanvas() {
    strokes = [];
    drawing = false;
    openStroke = false;
    hasInk = false;
    sampleCount = 0;
    redraw(draft.sensor_angle);
  }

  onMount(() => {
    sizeCanvas();
    const observer = new ResizeObserver(() => sizeCanvas());
    if (canvasEl) observer.observe(canvasEl);
    return () => observer.disconnect();
  });

  // Keep the corrected-axis guide in sync with the slider.
  $effect(() => {
    const angle = draft.sensor_angle;
    if (canvasEl) redraw(angle);
  });
</script>

<div class="flex-1 overflow-y-auto p-6 lg:p-8">
  <div class="mx-auto max-w-[860px] space-y-6">
    <!-- Header -->
    <div class="flex flex-wrap items-end justify-between gap-4 border-b border-edge/60 pb-4">
      <div>
        <h2 class="text-lg font-bold tracking-tight text-ink">Angle Adjustment</h2>
        <p class="text-xs text-ink-dim">
          Adjust the mouse sensor angle so that a mouse held at a slight tilt still tracks precisely on the horizontal axis.
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

    <!-- Explanatory Keychron callout -->
    <div class="flex items-start gap-3 rounded-2xl border border-glorious/35 bg-glorious/[0.07] px-5 py-4">
      <Info size={18} class="mt-0.5 shrink-0 text-glorious" />
      <div class="text-xs leading-relaxed text-ink-dim">
        <p class="font-bold text-glorious">
          Compensates for natural wrist and grip tilt during horizontal swipes.
        </p>
        <p class="mt-1">
          When drawing horizontal lines in games or productivity, a tilted grip naturally pulls the crosshair into a diagonal slope. Rotating the internal sensor angle aligns physical sweeps with true screen coordinates.
        </p>
      </div>
    </div>

    <!-- Sensor angle tuning card -->
    <section class="rounded-2xl border border-edge bg-card p-5">
      <div class="mb-4 flex items-start justify-between gap-4">
        <div>
          <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
            SENSOR ANGLE OFFSET
          </h3>
          <p class="text-xs text-ink-dim">
            Fine sensor angle rotation from {SENSOR_ANGLE_MIN}° to +{SENSOR_ANGLE_MAX}°.
          </p>
        </div>
        <span
          class="shrink-0 rounded-full border px-4 py-1 text-sm font-black tabular-nums transition {isNeutral
            ? 'border-edge bg-canvas text-ink-dim'
            : 'border-glorious/80 bg-zinc-900 text-glorious shadow-[0_0_14px_rgba(255,184,0,0.22)]'}"
          aria-live="polite"
        >
          {formatAngle(draft.sensor_angle)}
        </span>
      </div>

      <div class="relative py-2">
        <input
          type="range"
          min={SENSOR_ANGLE_MIN}
          max={SENSOR_ANGLE_MAX}
          step="1"
          value={draft.sensor_angle}
          style="--fill: {rangeFill(
            draft.sensor_angle,
            SENSOR_ANGLE_MIN,
            SENSOR_ANGLE_MAX,
          )}"
          oninput={(e) => setAngle(parseInt(e.currentTarget.value, 10))}
          class="w-full cursor-pointer"
          aria-label="Sensor angle in degrees"
        />
      </div>

      <div class="mt-2 flex justify-between text-[10px] font-semibold text-ink-faint">
        {#each ANGLE_TICKS as tick (tick)}
          <span class={tick === 0 ? 'text-glorious font-bold' : ''}>
            {tick > 0 ? `+${tick}°` : `${tick}°`}
          </span>
        {/each}
      </div>

      <div class="mt-5 flex flex-wrap items-center justify-between gap-3 pt-3 border-t border-white/[0.04]">
        <div class="flex flex-wrap gap-2">
          {#each ANGLE_PRESETS as preset (preset)}
            {@const isSelected = draft.sensor_angle === preset}
            <button
              type="button"
              class="rounded-full border px-3.5 py-1.5 text-xs font-bold tabular-nums transition cursor-pointer {isSelected
                ? 'border-glorious/80 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.22)] ring-1 ring-glorious/40'
                : 'border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
              aria-pressed={isSelected}
              onclick={() => setAngle(preset)}
            >
              {preset > 0 ? `+${preset}°` : `${preset}°`}
            </button>
          {/each}
        </div>

        <button
          type="button"
          disabled={isNeutral}
          onclick={() => setAngle(0)}
          class="flex items-center gap-1.5 rounded-full border border-edge bg-canvas px-4 py-1.5 text-xs font-semibold text-ink-dim transition hover:border-glorious/60 hover:text-glorious disabled:cursor-not-allowed disabled:opacity-40 cursor-pointer"
        >
          <RotateCcw size={13} />
          Reset to 0°
        </button>
      </div>
    </section>

    <!-- Keychron Before & After Live Tracking Comparison Canvas -->
    <section class="rounded-2xl border border-edge bg-card p-5">
      <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
        <div>
          <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">
            BEFORE & AFTER TRACKING TEST
          </h3>
          <p class="text-xs text-ink-dim">
            Sweep the pointer horizontally across the canvas to compare natural arm drift vs the corrected angle axis.
          </p>
        </div>
        <div class="flex items-center gap-2">
          <span class="flex items-center gap-1.5 rounded-full border border-edge bg-canvas px-3 py-1 text-[11px] font-semibold text-ink-dim tabular-nums">
            <Activity size={12} class="text-glorious" />
            {sampleCount} pts
          </span>
          <button
            type="button"
            disabled={!hasInk}
            onclick={clearCanvas}
            class="flex items-center gap-1.5 rounded-full border border-edge bg-canvas px-3.5 py-1 text-xs font-semibold text-ink-dim transition hover:border-edge-bright hover:text-ink disabled:cursor-not-allowed disabled:opacity-40 cursor-pointer"
          >
            <Eraser size={13} />
            Clear
          </button>
        </div>
      </div>

      <div class="relative overflow-hidden rounded-xl border border-edge bg-canvas">
        <canvas
          bind:this={canvasEl}
          aria-label="Live tracking test canvas — move the pointer here to draw your mouse trajectory"
          class="block h-[280px] w-full cursor-crosshair touch-none"
          onpointerenter={onPointerEnter}
          onpointerdown={onPointerDown}
          onpointermove={onPointerMove}
          onpointerleave={onPointerLeave}
        ></canvas>

        {#if !hasInk}
          <div class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-2 text-ink-faint">
            <Crosshair size={26} class="text-glorious/70" />
            <span class="text-xs font-semibold text-ink-dim">
              Sweep your mouse horizontally across this canvas
            </span>
            <span class="text-[10px] text-ink-faint">
              Observe how your natural wrist rotation compares to the gold corrected axis
            </span>
          </div>
        {/if}

        <div class="pointer-events-none absolute left-3 top-3 flex items-center gap-2 rounded-full border border-edge/80 bg-panel/85 px-2.5 py-1 text-[10px] font-semibold text-ink-dim backdrop-blur">
          <span class="h-1.5 w-1.5 rounded-full bg-glorious animate-pulse"></span>
          LIVE TRAJECTORY
        </div>

        <div class="pointer-events-none absolute bottom-3 left-3 flex flex-wrap items-center gap-3 text-[10px] font-semibold">
          <span class="flex items-center gap-1.5 text-glorious">
            <span class="h-0.5 w-4 rounded-full bg-glorious"></span>
            Corrected Axis ({formatAngle(draft.sensor_angle)})
          </span>
          <span class="flex items-center gap-1.5 text-ink-faint">
            <span class="h-0.5 w-4 rounded-full bg-ink-faint"></span>
            Raw Horizontal Axis (0°)
          </span>
        </div>
      </div>
    </section>
  </div>
</div>

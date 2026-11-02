<script lang="ts">
  import type { SettingsDraft } from '../lib/types';
  import { LIGHTING_EFFECTS, SLEEP_OPTIONS, clamp } from '../lib/actions';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
  }

  let { draft, onChange }: Props = $props();

  const currentEffect = $derived(
    LIGHTING_EFFECTS.find(
      (e) => e.id === draft.lighting.effect || e.name === (draft.lighting.effect as any)
    ) ??
      LIGHTING_EFFECTS.find((e) => e.id === 0) ??
      LIGHTING_EFFECTS[0]
  );

  const isOff = $derived(currentEffect.id === 0 || (currentEffect.name as string) === 'off');

  function setEffect(effectId: number) {
    onChange((prev) => ({
      ...prev,
      lighting: { ...prev.lighting, effect: effectId },
    }));
  }

  function setBrightness(brightness: number) {
    onChange((prev) => ({
      ...prev,
      lighting: { ...prev.lighting, brightness: clamp(brightness, 0, 100) },
    }));
  }

  function setRate(rate: number) {
    onChange((prev) => ({
      ...prev,
      lighting: { ...prev.lighting, rate: clamp(rate, 0, 100) },
    }));
  }

  function setColor(color: string) {
    onChange((prev) => ({
      ...prev,
      lighting: { ...prev.lighting, color },
    }));
  }

  function setSleep(seconds: number | 'off') {
    onChange((prev) => ({
      ...prev,
      sleep_seconds: seconds,
    }));
  }
</script>

<div class="flex-1 overflow-y-auto p-8">
  <div class="mx-auto max-w-[800px] space-y-8">
    <!-- Header -->
    <div>
      <h2 class="text-xl font-bold text-ink">Lighting & Power</h2>
      <p class="text-xs text-ink-dim">Customize RGB LED effects, brightness levels, animation speeds, and wireless power-saving sleep timer.</p>
    </div>

    <!-- Effects Cards Grid -->
    <section class="rounded-2xl border border-edge bg-card p-6">
      <div class="mb-4">
        <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">LIGHTING EFFECT</h3>
        <p class="mt-1 text-xs text-ink-dim">Choose an active RGB pattern for the chassis diffusers and scroll wheel.</p>
      </div>

      <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
        {#each LIGHTING_EFFECTS as effect (effect.id)}
          {@const isSelected =
            draft.lighting.effect === effect.id ||
            (draft.lighting.effect as any) === effect.name ||
            (effect.id === 0 && ((draft.lighting.effect as any) === 'off' || draft.lighting.effect === 0))}
          <button
            type="button"
            class="flex flex-col rounded-xl border p-3.5 text-left transition {isSelected
              ? 'border-glorious bg-zinc-900 ring-2 ring-glorious/60 shadow-[0_0_16px_rgba(255,184,0,0.25)]'
              : 'border-edge bg-canvas/50 hover:border-edge-bright hover:bg-card-hover'}"
            onclick={() => setEffect(effect.id)}
          >
            <div class="flex items-center justify-between mb-2">
              <span class="text-xs font-bold {isSelected ? 'text-glorious' : 'text-ink'}">{effect.label}</span>
              {#if isSelected}
                <span class="h-2 w-2 rounded-full bg-glorious shadow-[0_0_6px_#ffb800]"></span>
              {/if}
            </div>
            <div
              class="h-2 w-full rounded-full border border-edge/60"
              style={effect.id === 0
                ? 'background: #18181b;'
                : effect.preview
                  ? `background: ${effect.preview}`
                  : effect.colored
                    ? `background: ${draft.lighting.color}`
                    : 'background: #2a2a33'}
            ></div>
          </button>
        {/each}
      </div>
    </section>

    <!-- Controls Row: Brightness & Speed -->
    <div class="grid grid-cols-1 gap-6 md:grid-cols-2 {isOff ? 'opacity-40 pointer-events-none' : ''}">
      <!-- Brightness Slider -->
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">BRIGHTNESS</h3>
            <p class="mt-1 text-xs text-ink-dim">
              {isOff ? 'Disabled (Lighting is Off)' : 'LED luminance output'}
            </p>
          </div>
          <span class="rounded-full border border-edge bg-canvas px-3 py-1 text-xs font-bold text-glorious">
            {isOff ? '0%' : `${draft.lighting.brightness}%`}
          </span>
        </div>

        <input
          type="range"
          min="0"
          max="100"
          step="1"
          disabled={isOff}
          value={draft.lighting.brightness}
          style="--fill: {draft.lighting.brightness}%"
          oninput={(e) => setBrightness(parseInt(e.currentTarget.value, 10))}
          class="w-full"
        />
        <div class="mt-2 flex justify-between text-[10px] text-ink-faint">
          <span>0% (Off)</span>
          <span>100% (Max)</span>
        </div>
      </section>

      <!-- Speed / Rate Slider -->
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">ANIMATION SPEED</h3>
            <p class="mt-1 text-xs text-ink-dim">
              {isOff ? 'Disabled (Lighting is Off)' : 'Cycle and transition frequency'}
            </p>
          </div>
          <span class="rounded-full border border-edge bg-canvas px-3 py-1 text-xs font-bold text-glorious">
            {draft.lighting.rate}%
          </span>
        </div>

        <input
          type="range"
          min="0"
          max="100"
          step="1"
          disabled={isOff}
          value={draft.lighting.rate}
          style="--fill: {draft.lighting.rate}%"
          oninput={(e) => setRate(parseInt(e.currentTarget.value, 10))}
          class="w-full"
        />
        <div class="mt-2 flex justify-between text-[10px] text-ink-faint">
          <span>0% (Slowest)</span>
          <span>100% (Fastest)</span>
        </div>
      </section>
    </div>

    <!-- Color Picker (when effect uses color and lighting is not off) -->
    {#if currentEffect.colored && !isOff}
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4">
          <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">COLOR SELECTION</h3>
          <p class="mt-1 text-xs text-ink-dim">Active color for {currentEffect.label}</p>
        </div>

        <div class="flex items-center gap-4">
          <div class="relative flex items-center">
            <input
              type="color"
              value={draft.lighting.color}
              oninput={(e) => setColor(e.currentTarget.value)}
              class="h-10 w-10 cursor-pointer rounded-xl border border-edge bg-transparent p-0"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs text-ink-faint">Hex:</span>
            <input
              type="text"
              value={draft.lighting.color}
              oninput={(e) => setColor(e.currentTarget.value)}
              class="w-28 rounded-lg border border-edge bg-canvas px-2.5 py-1 text-xs font-mono font-bold text-ink focus:border-glorious focus:outline-none uppercase"
            />
          </div>
        </div>
      </section>
    {/if}

    <!-- Sleep Timer Segmented Options -->
    <section class="rounded-2xl border border-edge bg-card p-6">
      <div class="mb-4">
        <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">SLEEP TIMER</h3>
        <p class="mt-1 text-xs text-ink-dim">Put mouse into wireless low-power sleep mode after inactivity.</p>
      </div>

      <div class="flex flex-wrap gap-2.5">
        {#each SLEEP_OPTIONS as opt (opt.label)}
          {@const isSelected = draft.sleep_seconds === opt.seconds}
          <button
            type="button"
            class="rounded-full px-4 py-2 text-xs font-bold transition {isSelected
              ? 'border border-glorious/60 bg-zinc-900 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.25)]'
              : 'border border-edge bg-canvas text-ink-dim hover:border-edge-bright hover:text-ink'}"
            onclick={() => setSleep(opt.seconds)}
          >
            {opt.label}
          </button>
        {/each}
      </div>
    </section>
  </div>
</div>

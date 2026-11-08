<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { SettingsDraft, LightingSettings } from '../lib/types';
  import { LIGHTING_EFFECTS, SLEEP_OPTIONS, clamp } from '../lib/actions';
  import { i18n } from '../lib/i18n.svelte';

  interface Props {
    draft: SettingsDraft;
    onChange: (updater: (draft: SettingsDraft) => SettingsDraft) => void;
    onLivePreview?: (lighting: LightingSettings) => void;
  }

  let { draft, onChange, onLivePreview }: Props = $props();

  const currentEffect = $derived(
    LIGHTING_EFFECTS.find(
      (e) => e.id === draft.lighting.effect || e.name === (draft.lighting.effect as any)
    ) ??
      LIGHTING_EFFECTS.find((e) => e.id === 0) ??
      LIGHTING_EFFECTS[0]
  );

  const isOff = $derived(currentEffect.id === 0 || (currentEffect.name as string) === 'off');

  function getEffectLabel(eff: { name: string; label: string }): string {
    const key = `lighting.effect.${eff.name}`;
    const translated = i18n.t(key);
    return translated !== key ? translated : eff.label;
  }
  const currentEffectLabel = $derived(getEffectLabel(currentEffect));

  // Dynamic animation duration: 100% speed -> 0.5s, 0% speed -> 5.0s
  const animDuration = $derived(Math.max(0.5, (100 - draft.lighting.rate) / 20));
  // Brightness scales glow opacity: 100% -> 1.0, 0% -> 0.15 (or 0 when off)
  const glowOpacity = $derived(isOff ? 0.08 : Math.max(0.15, draft.lighting.brightness / 100));

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function triggerLivePreview(lighting: LightingSettings, debounce = false) {
    if (debounce) {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        onLivePreview?.(lighting);
      }, 120);
    } else {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
        debounceTimer = null;
      }
      onLivePreview?.(lighting);
    }
  }

  onDestroy(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
  });

  function setEffect(effectId: number) {
    const nextLighting = { ...draft.lighting, effect: effectId };
    onChange((prev) => ({
      ...prev,
      lighting: nextLighting,
    }));
    triggerLivePreview(nextLighting, false);
  }

  function setBrightness(brightness: number) {
    const clamped = clamp(brightness, 0, 100);
    const nextLighting = { ...draft.lighting, brightness: clamped };
    onChange((prev) => ({
      ...prev,
      lighting: nextLighting,
    }));
    triggerLivePreview(nextLighting, true);
  }

  function setRate(rate: number) {
    const clamped = clamp(rate, 0, 100);
    const nextLighting = { ...draft.lighting, rate: clamped };
    onChange((prev) => ({
      ...prev,
      lighting: nextLighting,
    }));
    triggerLivePreview(nextLighting, true);
  }

  function setColor(color: string) {
    const nextLighting = { ...draft.lighting, color };
    onChange((prev) => ({
      ...prev,
      lighting: nextLighting,
    }));
    triggerLivePreview(nextLighting, false);
  }

  function setSleep(seconds: number | 'off') {
    onChange((prev) => ({
      ...prev,
      sleep_seconds: seconds,
    }));
  }

  function getEffectClass(effectId: number): string {
    switch (effectId) {
      case 1:
        return 'anim-glorious';
      case 2:
        return 'anim-seamless';
      case 3:
        return 'anim-breathing';
      case 4:
        return 'anim-static';
      case 5:
        return 'anim-single-breathing';
      case 6:
        return 'anim-tail';
      case 7:
        return 'anim-rave';
      case 8:
        return 'anim-wave';
      case 0:
      default:
        return 'anim-off';
    }
  }

  function getEffectStyle(effectId: number, color: string): string {
    switch (effectId) {
      case 1: // Glorious Mode
        return 'background: linear-gradient(180deg, #ff0000, #ff8800, #ffff00, #00ff00, #00ffff, #0066ff, #9900ff, #ff0000); background-size: 100% 200%; box-shadow: 0 0 14px rgba(255, 184, 0, 0.45);';
      case 2: // Seamless Breathing
        return 'background: linear-gradient(135deg, #ff2d55, #ffb800, #2dff88, #00d4ff, #b52dff); background-size: 150% 150%; box-shadow: 0 0 14px rgba(0, 212, 255, 0.4);';
      case 3: // Breathing
        return `background: ${color}; box-shadow: 0 0 16px ${color};`;
      case 4: // Single Color (Static)
        return `background: ${color}; box-shadow: 0 0 16px ${color};`;
      case 5: // Single Breathing
        return `background: ${color}; box-shadow: 0 0 16px ${color};`;
      case 6: // Tail
        return 'background: linear-gradient(180deg, transparent 0%, #fff60a 20%, #0aff7d 40%, #ff600a 60%, #0affe5 80%, #ff0ad4 90%, #ff0000 100%); background-size: 100% 300%; box-shadow: 0 0 14px rgba(255, 184, 0, 0.45);';
      case 7: // Rave
        return 'box-shadow: 0 0 16px rgba(255, 0, 85, 0.6);';
      case 8: // Wave
        return 'background: linear-gradient(180deg, #ff2d2d, #ffb800, #2dff6a, #2d8cff, #c12dff, #ff2d2d); background-size: 100% 200%; box-shadow: 0 0 14px rgba(45, 140, 255, 0.45);';
      case 0:
      default:
        return 'background: #18181b; box-shadow: none;';
    }
  }

  function getCardClass(effectId: number): string {
    switch (effectId) {
      case 1:
        return 'card-anim-glorious';
      case 2:
        return 'card-anim-seamless';
      case 3:
        return 'anim-breathing';
      case 4:
        return 'anim-static';
      case 5:
        return 'anim-single-breathing';
      case 6:
        return 'card-anim-tail';
      case 7:
        return 'anim-rave';
      case 8:
        return 'card-anim-wave';
      case 0:
      default:
        return 'anim-off';
    }
  }

  function getCardStyle(effectId: number, color: string): string {
    switch (effectId) {
      case 1: // Glorious Mode (horizontal flow)
        return 'background: linear-gradient(90deg, #ff0000, #ff8800, #ffff00, #00ff00, #00ffff, #0066ff, #9900ff, #ff0000); background-size: 200% 100%; box-shadow: 0 0 14px rgba(255, 184, 0, 0.45);';
      case 2: // Seamless Breathing
        return 'background: linear-gradient(90deg, #ff2d55, #ffb800, #2dff88, #00d4ff, #b52dff); background-size: 150% 150%; box-shadow: 0 0 14px rgba(0, 212, 255, 0.4);';
      case 3: // Breathing
        return `background: ${color}; box-shadow: 0 0 16px ${color};`;
      case 4: // Single Color (Static)
        return `background: ${color}; box-shadow: 0 0 16px ${color};`;
      case 5: // Single Breathing
        return `background: ${color}; box-shadow: 0 0 16px ${color};`;
      case 6: // Tail (horizontal flow)
        return 'background: linear-gradient(90deg, transparent 0%, #fff60a 20%, #0aff7d 40%, #ff600a 60%, #0affe5 80%, #ff0ad4 90%, #ff0000 100%); background-size: 300% 100%; box-shadow: 0 0 14px rgba(255, 184, 0, 0.45);';
      case 7: // Rave
        return 'box-shadow: 0 0 16px rgba(255, 0, 85, 0.6);';
      case 8: // Wave (horizontal flow)
        return 'background: linear-gradient(90deg, #ff2d2d, #ffb800, #2dff6a, #2d8cff, #c12dff, #ff2d2d); background-size: 200% 100%; box-shadow: 0 0 14px rgba(45, 140, 255, 0.45);';
      case 0:
      default:
        return 'background: #18181b; box-shadow: none;';
    }
  }
</script>

<div class="flex-1 overflow-y-auto p-8">
  <div class="mx-auto max-w-[800px] space-y-8">
    <!-- Header -->
    <div>
      <h2 class="text-xl font-bold text-ink">Lighting & Power</h2>
      <p class="text-xs text-ink-dim">Customize RGB LED effects, brightness levels, animation speeds, and wireless power-saving sleep timer.</p>
    </div>

    <!-- Top Hero / Mouse Diffuser Silhouette Preview -->
    <section class="relative overflow-hidden rounded-2xl border border-edge bg-gradient-to-b from-card via-card/90 to-canvas p-6 shadow-xl">
      <!-- Background Ambient Glow -->
      <div
        class="absolute -top-12 left-1/2 -translate-x-1/2 h-56 w-80 rounded-full blur-3xl pointer-events-none transition-all duration-500"
        style="background: {isOff ? 'transparent' : (currentEffect.colored ? draft.lighting.color : '#ffb800')}; opacity: {isOff ? 0 : glowOpacity * 0.35};"
      ></div>

      <div class="relative z-10 flex flex-col md:flex-row items-center justify-between gap-6">
        <!-- Info Column -->
        <div class="space-y-3 text-center md:text-left">
          <div class="inline-flex items-center gap-2 rounded-full border border-glorious/30 bg-zinc-900/80 px-3 py-1 text-[11px] font-bold text-glorious shadow-[0_0_12px_rgba(255,184,0,0.2)]">
            <span class="h-2 w-2 rounded-full {isOff ? 'bg-zinc-600' : 'bg-emerald-400 animate-pulse'}"></span>
            <span>{isOff ? i18n.t('lighting.leds_off') : i18n.t('lighting.sync_live')}</span>
          </div>
          <div>
            <h3 class="text-lg font-black tracking-wide text-ink">{currentEffectLabel}</h3>
            <p class="text-xs text-ink-dim">{i18n.t('lighting.hero_desc')}</p>
          </div>
          <div class="flex flex-wrap items-center justify-center md:justify-start gap-2 pt-1">
            <span class="rounded-lg border border-edge bg-canvas/80 px-2.5 py-1 text-[11px] font-semibold text-ink-dim">
              {i18n.t('lighting.brightness_label')} <strong class="text-ink">{isOff ? '0%' : `${draft.lighting.brightness}%`}</strong>
            </span>
            <span class="rounded-lg border border-edge bg-canvas/80 px-2.5 py-1 text-[11px] font-semibold text-ink-dim">
              {i18n.t('lighting.speed_label')} <strong class="text-ink">{isOff ? '0%' : `${draft.lighting.rate}% (${animDuration.toFixed(1)}s)`}</strong>
            </span>
            {#if currentEffect.colored && !isOff}
              <span class="inline-flex items-center gap-1.5 rounded-lg border border-edge bg-canvas/80 px-2.5 py-1 text-[11px] font-semibold text-ink-dim">
                <span class="h-2.5 w-2.5 rounded-full" style="background: {draft.lighting.color}; box-shadow: 0 0 6px {draft.lighting.color};"></span>
                <strong class="font-mono text-ink uppercase">{draft.lighting.color}</strong>
              </span>
            {/if}
          </div>
        </div>

        <!-- Mouse Diffuser Silhouette Visualizer -->
        <div class="relative flex h-48 w-40 shrink-0 items-center justify-center">
          <!-- Diffuser Underglow Floor -->
          <div
            class="absolute inset-0 rounded-[44px] blur-xl transition-all duration-300 pointer-events-none"
            style="background: {isOff ? 'transparent' : (currentEffect.colored ? draft.lighting.color : '#ffb800')}; opacity: {isOff ? 0 : glowOpacity * 0.45};"
          ></div>

          <!-- Mouse Chassis Silhouette Body -->
          <div class="relative h-44 w-28 rounded-[38px] border border-zinc-700/80 bg-[#0d0d12] shadow-2xl flex flex-col items-center overflow-hidden">
            <!-- Center Button Split Line -->
            <div class="absolute top-0 h-14 w-px bg-zinc-800/80"></div>

            <!-- Scroll Wheel Cavity & Illuminated Wheel -->
            <div class="relative z-10 mt-3.5 flex h-10 w-4 items-center justify-center rounded-md border border-zinc-700/90 bg-zinc-900 shadow-inner">
              <!-- Scroll Wheel LED -->
              <div
                class="h-6 w-2 rounded-sm transition-all duration-200 {getEffectClass(draft.lighting.effect)}"
                style="opacity: {glowOpacity}; --anim-duration: {animDuration}s; --effect-color: {draft.lighting.color}; {getEffectStyle(draft.lighting.effect, draft.lighting.color)}"
              ></div>
            </div>

            <!-- Honeycomb Palm Accents (Model O Signature Aesthetic) -->
            <div class="mt-4 flex flex-col items-center gap-1 opacity-25">
              <div class="flex gap-1.5">
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
              </div>
              <div class="flex gap-1.5">
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
              </div>
              <div class="flex gap-1.5">
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
                <span class="h-1.5 w-1.5 rounded-full bg-zinc-500"></span>
              </div>
            </div>

            <!-- Left Flank Diffuser LED Strip -->
            <div
              class="absolute left-1 top-10 bottom-8 w-1.5 rounded-full transition-all duration-200 {getEffectClass(draft.lighting.effect)}"
              style="opacity: {glowOpacity}; --anim-duration: {animDuration}s; --effect-color: {draft.lighting.color}; {getEffectStyle(draft.lighting.effect, draft.lighting.color)}"
            ></div>

            <!-- Right Flank Diffuser LED Strip -->
            <div
              class="absolute right-1 top-10 bottom-8 w-1.5 rounded-full transition-all duration-200 {getEffectClass(draft.lighting.effect)}"
              style="opacity: {glowOpacity}; --anim-duration: {animDuration}s; --effect-color: {draft.lighting.color}; {getEffectStyle(draft.lighting.effect, draft.lighting.color)}"
            ></div>
          </div>
        </div>
      </div>
    </section>

    <!-- Effects Cards Grid -->
    <section class="rounded-2xl border border-edge bg-card p-6">
      <div class="mb-4">
        <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">{i18n.t('lighting.effect_header')}</h3>
        <p class="mt-1 text-xs text-ink-dim">{i18n.t('lighting.effect_desc')}</p>
      </div>

      <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
        {#each LIGHTING_EFFECTS as effect (effect.id)}
          {@const isSelected =
            draft.lighting.effect === effect.id ||
            (draft.lighting.effect as any) === effect.name ||
            (effect.id === 0 && ((draft.lighting.effect as any) === 'off' || draft.lighting.effect === 0))}
          <button
            type="button"
            class="group flex flex-col rounded-xl border p-3.5 text-left transition {isSelected
              ? 'border-glorious bg-zinc-900 ring-2 ring-glorious/60 shadow-[0_0_18px_rgba(255,184,0,0.25)]'
              : 'border-edge bg-canvas/50 hover:border-edge-bright hover:bg-card-hover'}"
            onclick={() => setEffect(effect.id)}
          >
            <div class="flex items-center justify-between mb-2.5">
              <span class="text-xs font-bold {isSelected ? 'text-glorious' : 'text-ink'}">{getEffectLabel(effect)}</span>
              {#if isSelected}
                <span class="h-2 w-2 rounded-full bg-glorious shadow-[0_0_6px_#ffb800]"></span>
              {/if}
            </div>

            <!-- Animated Preview Strip -->
            <div class="relative h-3 w-full overflow-hidden rounded-full border border-edge/60 bg-zinc-950">
              {#if effect.id === 0}
                <div class="flex h-full w-full items-center justify-center bg-zinc-900 text-[9px] font-bold tracking-wider text-ink-faint uppercase">
                  {i18n.t('lighting.leds_inactive')}
                </div>
              {:else}
                <div
                  class="h-full w-full rounded-full transition-all duration-200 {getCardClass(effect.id)}"
                  style="opacity: {glowOpacity}; --anim-duration: {animDuration}s; --effect-color: {draft.lighting.color}; {getCardStyle(effect.id, draft.lighting.color)}"
                ></div>
              {/if}
            </div>
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
            <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">{i18n.t('lighting.brightness_section')}</h3>
            <p class="mt-1 text-xs text-ink-dim">
              {isOff ? i18n.t('lighting.brightness_disabled') : i18n.t('lighting.brightness_desc')}
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
          <span>0% ({i18n.t('lighting.effect.off')})</span>
          <span>100% (Max)</span>
        </div>
      </section>

      <!-- Speed / Rate Slider -->
      <section class="rounded-2xl border border-edge bg-card p-6">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">{i18n.t('lighting.speed_section')}</h3>
            <p class="mt-1 text-xs text-ink-dim">
              {isOff ? i18n.t('lighting.brightness_disabled') : i18n.t('lighting.speed_desc')}
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
          <span>0%</span>
          <span>100%</span>
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
        <h3 class="text-xs font-bold tracking-[0.18em] text-ink-faint">{i18n.t('lighting.sleep_section')}</h3>
        <p class="mt-1 text-xs text-ink-dim">{i18n.t('lighting.sleep_desc')}</p>
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
            {opt.seconds === 'off' ? i18n.t('lighting.sleep_off') : opt.label}
          </button>
        {/each}
      </div>
    </section>
  </div>
</div>

<style>
  /* ── Hero Preview (Vertical Top-to-Bottom Flow) ── */
  @keyframes rainbow-flow {
    from {
      background-position: 50% 200%;
    }
    to {
      background-position: 50% 0%;
    }
  }

  @keyframes hue-breathe {
    0%, 100% {
      opacity: 0.35;
      filter: hue-rotate(0deg) brightness(0.7);
    }
    50% {
      opacity: 1;
      filter: hue-rotate(180deg) brightness(1.25);
    }
  }

  @keyframes single-breathe {
    0%, 100% {
      opacity: 0.2;
      transform: scale(0.97);
    }
    50% {
      opacity: 1;
      transform: scale(1);
    }
  }

  @keyframes pulse-effect {
    0%, 100% {
      opacity: 0.25;
      filter: brightness(0.6) drop-shadow(0 0 2px var(--effect-color, #ffb800));
    }
    50% {
      opacity: 1;
      filter: brightness(1.3) drop-shadow(0 0 12px var(--effect-color, #ffb800));
    }
  }

  @keyframes tail-flow {
    from {
      background-position: 50% 300%;
    }
    to {
      background-position: 50% 0%;
    }
  }

  @keyframes rave-strobe {
    0%, 100% {
      background-color: #ff0055;
      box-shadow: 0 0 14px #ff0055;
    }
    33% {
      background-color: #ffb800;
      box-shadow: 0 0 14px #ffb800;
    }
    66% {
      background-color: #00e5ff;
      box-shadow: 0 0 14px #00e5ff;
    }
  }

  @keyframes wave-slide {
    from {
      background-position: 50% 200%;
    }
    to {
      background-position: 50% 0%;
    }
  }

  .anim-glorious {
    animation: rainbow-flow var(--anim-duration, 2.5s) linear infinite;
  }

  .anim-seamless {
    animation: hue-breathe var(--anim-duration, 2.5s) ease-in-out infinite;
  }

  .anim-breathing {
    animation: single-breathe var(--anim-duration, 2.5s) ease-in-out infinite;
  }

  .anim-static {
    transition: box-shadow 0.2s ease-in-out;
  }

  .anim-single-breathing {
    animation: pulse-effect var(--anim-duration, 2.5s) ease-in-out infinite;
  }

  .anim-tail {
    animation: tail-flow var(--anim-duration, 2.5s) ease-in-out infinite;
  }

  .anim-rave {
    animation: rave-strobe var(--anim-duration, 2.5s) linear infinite;
  }

  .anim-wave {
    animation: wave-slide var(--anim-duration, 2.5s) linear infinite;
  }

  .anim-off {
    background: #18181b;
  }

  /* ── Card Preview (Horizontal Left-to-Right Flow) ── */
  @keyframes card-rainbow-flow {
    from {
      background-position: 200% 50%;
    }
    to {
      background-position: 0% 50%;
    }
  }

  @keyframes card-tail-flow {
    from {
      background-position: 300% 50%;
    }
    to {
      background-position: 0% 50%;
    }
  }

  @keyframes card-wave-slide {
    from {
      background-position: 200% 50%;
    }
    to {
      background-position: 0% 50%;
    }
  }

  .card-anim-glorious {
    animation: card-rainbow-flow var(--anim-duration, 2.5s) linear infinite;
  }

  .card-anim-seamless {
    animation: hue-breathe var(--anim-duration, 2.5s) ease-in-out infinite;
  }

  .card-anim-tail {
    animation: card-tail-flow var(--anim-duration, 2.5s) ease-in-out infinite;
  }

  .card-anim-wave {
    animation: card-wave-slide var(--anim-duration, 2.5s) linear infinite;
  }
</style>

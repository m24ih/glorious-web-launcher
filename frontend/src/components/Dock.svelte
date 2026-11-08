<script lang="ts">
  import type { DeviceCapabilities, PinId, ActionCategory, PinAction } from '../lib/types';
  import { DOCK_CATEGORIES, DOCK_ACTIONS, PIN_LABELS, DEFAULT_PIN_ACTIONS, loadMacros } from '../lib/actions';
  import { RotateCcw } from 'lucide-svelte';
  import { i18n } from '../lib/i18n.svelte';

  interface Props {
    selectedPin: PinId | null;
    mappings: PinAction[];
    onAssign: (category: ActionCategory, action: string) => void;
    onResetPin?: (pin: PinId) => void;
    leftClickLocked?: boolean;
    caps?: DeviceCapabilities;
  }

  let { selectedPin, mappings, onAssign, onResetPin, leftClickLocked = false, caps }: Props = $props();

  let activeCategory = $state<ActionCategory>('mousekeys');
  let shortcutFilter = $state<'all' | 'win' | 'mac'>('all');

  // Turbo local state
  let turboCount = $state(1);
  let turboRate = $state(50);
  let turboKey = $state('A');
  let testInput = $state('');

  // Combinations local state
  let comboMods = $state({ ctrl: false, shift: false, alt: false, win: false });
  let comboTarget = $state('S');

  const profileActions = $derived(
    (DOCK_ACTIONS['profile'] ?? []).filter((a) => {
      const m = /^profile:(\d+)$/i.exec(a.id);
      if (!m) return true;
      const n = parseInt(m[1], 10);
      return n <= (caps?.maxProfiles ?? 5);
    })
  );

  const categoryActions = $derived(
    activeCategory === 'profile' ? profileActions : (DOCK_ACTIONS[activeCategory] ?? [])
  );

  const currentMapping = $derived(
    selectedPin ? mappings.find((m) => m.pin === selectedPin) : null
  );

  function getPinLabel(pin: PinId): string {
    const key = `stage.${pin}`;
    const translated = i18n.t(key);
    return translated !== key ? translated : PIN_LABELS[pin];
  }

  const currentLabel = $derived(
    selectedPin ? getPinLabel(selectedPin) : null
  );

  function getActionLabel(act: { id: string; label: string }): string {
    const key = `dock.action.${act.id}`;
    const translated = i18n.t(key);
    return translated !== key ? translated : act.label;
  }

  function getMappedActionName(actionStr: string): string {
    const found = Object.values(DOCK_ACTIONS).flat().find((a) => a.label === actionStr || a.id === actionStr);
    if (found) {
      return getActionLabel(found);
    }
    return actionStr;
  }

  // Load macros when macro category selected to allow direct selection
  let macroList = $derived(activeCategory === 'macro' ? loadMacros() : []);

  const defaultAction = $derived(
    selectedPin ? DEFAULT_PIN_ACTIONS.find((m) => m.pin === selectedPin) : null
  );

  const isCustomized = $derived(
    Boolean(
      selectedPin &&
        currentMapping &&
        defaultAction &&
        currentMapping.action !== defaultAction.action
    )
  );

  function resetCurrentButton() {
    if (!selectedPin) return;
    if (onResetPin) {
      onResetPin(selectedPin);
    } else {
      const def = DEFAULT_PIN_ACTIONS.find((m) => m.pin === selectedPin);
      if (def) {
        onAssign('mousekeys', def.action);
      }
    }
  }

  const isLocked = $derived(
    leftClickLocked && selectedPin === 'left'
  );

  function applyTurbo() {
    if (!selectedPin || isLocked) return;
    const action = `turbo:${turboKey}:${turboCount}:${turboRate}`;
    onAssign('gaming', action);
  }

  function assignCombination() {
    if (!selectedPin || isLocked) return;
    const parts: string[] = [];
    if (comboMods.ctrl) parts.push('Ctrl');
    if (comboMods.shift) parts.push('Shift');
    if (comboMods.alt) parts.push('Alt');
    if (comboMods.win) parts.push('Win');
    parts.push(comboTarget);
    onAssign('combinations', parts.join('+'));
  }

  function previewCombo(): string {
    const parts: string[] = [];
    if (comboMods.ctrl) parts.push('Ctrl');
    if (comboMods.shift) parts.push('Shift');
    if (comboMods.alt) parts.push('Alt');
    if (comboMods.win) parts.push('Win');
    parts.push(comboTarget);
    return parts.join('+');
  }

  function filteredShortcuts() {
    const actions = DOCK_ACTIONS['shortcuts'] ?? [];
    if (shortcutFilter === 'all') return actions;
    if (shortcutFilter === 'win') {
      return actions.filter((a) => !a.id.startsWith('sc:cmd'));
    }
    return actions.filter((a) => a.id.startsWith('sc:cmd') || a.id.startsWith('sc:ctrl'));
  }
</script>

<div class="border-t border-edge bg-panel px-4 py-3">
  <div class="mx-auto flex max-w-[1240px] flex-col gap-2.5">
    <!-- Category tabs row + reset button -->
    <div class="flex flex-wrap items-center justify-between gap-2">
      <!-- Tabs -->
      <div class="flex flex-wrap gap-1">
        {#each DOCK_CATEGORIES as cat (cat.id)}
          <button
            type="button"
            class="rounded px-3 py-1 text-[11px] font-semibold transition {activeCategory === cat.id
              ? 'border border-[#6277ff] bg-[#4657c9] text-white'
              : 'border border-transparent text-ink-dim hover:bg-card-hover hover:text-ink'}"
            onclick={() => (activeCategory = cat.id)}
            aria-pressed={activeCategory === cat.id}
          >
            {i18n.t(`dock.category.${cat.id}`)}
          </button>
        {/each}
      </div>

      <!-- Right actions: Reset Button + Button Rebind Capsule -->
      <div class="flex items-center gap-2">
        {#if selectedPin && isCustomized}
          <button
            type="button"
            class="flex items-center gap-1.5 rounded border border-amber-500/40 bg-amber-950/30 px-3 py-1 text-[11px] font-bold text-amber-300 transition hover:border-amber-400 hover:bg-amber-900/50 hover:text-white"
            onclick={resetCurrentButton}
            title="{i18n.t('dock.reset_button')}: {currentLabel}"
          >
            <RotateCcw size={12} class="text-amber-400" />
            <span>{i18n.t('dock.reset_button')}</span>
          </button>
        {/if}

        <!-- Button Rebind Capsule -->
        {#if selectedPin && currentLabel}
          <div class="flex items-center gap-2 rounded border border-[#6277ff]/60 bg-[#4657c9]/15 px-3 py-1 text-[11px] font-medium text-[#aeb8ff]">
            <span class="h-1.5 w-1.5 rounded-full bg-[#6277ff]"></span>
            <span>{i18n.t('dock.editing', { label: currentLabel, action: getMappedActionName(currentMapping?.action ?? currentLabel) })}</span>
          </div>
        {:else}
          <div class="flex items-center gap-2 rounded border border-edge bg-card px-3 py-1 text-[11px] text-ink-faint">
            <span>{i18n.t('dock.select_prompt')}</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Lock banner for Left Click -->
    {#if isLocked}
      <div role="status" class="rounded border border-amber-500/35 bg-amber-950/40 px-3 py-2 text-xs font-medium text-amber-300">
        <span aria-hidden="true">🔒</span>
        <span class="ml-1.5">{i18n.t('dock.left_click_locked')}</span>
      </div>
    {/if}

    <!-- Action pills grid -->
    <div class="rounded-md border border-edge bg-card p-2.5">
      <!-- Gaming Turbo panel -->
      {#if activeCategory === 'gaming'}
        <div class="space-y-4">
          <div class="flex items-center gap-3">
            <h4 class="text-xs font-black tracking-[0.12em] text-ink">GAMING TURBO / RAPID FIRE</h4>
            <span class="rounded bg-glorious/10 px-2 py-0.5 text-[10px] font-bold text-glorious border border-glorious/30">LIVE</span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- Count -->
            <div class="rounded border border-edge bg-canvas/60 p-3">
              <label for="turbo-count" class="mb-1 block text-[11px] font-bold text-ink-dim">Repeat Count</label>
              <input
                id="turbo-count"
                type="number"
                min="0"
                max="9999"
                value={turboCount}
                oninput={(e) => turboCount = Math.max(0, Math.min(9999, parseInt(e.currentTarget.value || '0', 10)))}
                class="w-full rounded-lg border border-edge bg-card px-2 py-1.5 text-sm font-bold text-ink focus:border-glorious focus:outline-none"
              />
              <p class="mt-1 text-[10px] text-ink-faint">0 = Unlimited</p>
            </div>
            <!-- Rate -->
            <div class="rounded border border-edge bg-canvas/60 p-3">
              <label for="turbo-rate" class="mb-1 block text-[11px] font-bold text-ink-dim">Repeat Rate (ms)</label>
              <div class="flex items-center gap-2">
                <input
                  type="range"
                  id="turbo-rate"
                  min="10"
                  max="1000"
                  step="10"
                  value={turboRate}
                  oninput={(e) => turboRate = parseInt(e.currentTarget.value, 10)}
                  class="w-full accent-glorious"
                />
                <span class="w-12 text-xs font-bold text-glorious">{turboRate}ms</span>
              </div>
              <p class="mt-1 text-[10px] text-ink-faint">10ms - 1000ms</p>
            </div>
            <!-- Target key -->
            <div class="rounded border border-edge bg-canvas/60 p-3">
              <span class="mb-1 block text-[11px] font-bold text-ink-dim">Target Key</span>
              <div class="flex flex-wrap gap-1.5">
                {#each ['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z'] as k}
                  <button
                    type="button"
                    onclick={() => turboKey = k}
                    aria-pressed={turboKey === k}
                    class="rounded border px-2 py-1 text-xs font-bold transition {turboKey === k ? 'border-[#6277ff] bg-[#4657c9] text-white' : 'border-edge bg-card text-ink hover:border-[#6277ff]'}"
                  >
                    {k}
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <button
            type="button"
            onclick={applyTurbo}
            disabled={isLocked || !selectedPin}
            class="rounded bg-glorious px-5 py-2 text-xs font-extrabold text-canvas hover:bg-glorious-soft transition disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Apply Turbo
          </button>
        </div>
      {:else if activeCategory === 'combinations'}
        <div class="space-y-4">
          <div class="flex items-center gap-3">
            <h4 class="text-xs font-black tracking-[0.12em] text-ink">COMBINATION BUILDER</h4>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Modifiers -->
            <div class="rounded border border-edge bg-canvas/60 p-3">
              <h5 class="mb-2 text-[11px] font-bold text-ink-dim">Modifiers</h5>
              <div class="flex flex-wrap gap-2">
                {#each [
                  { id: 'ctrl', label: 'Ctrl' },
                  { id: 'shift', label: 'Shift' },
                  { id: 'alt', label: 'Alt' },
                  { id: 'win', label: 'Win / Cmd' },
                ] as mod}
                  <button
                    type="button"
                    onclick={() => comboMods = { ...comboMods, [mod.id]: !comboMods[mod.id as keyof typeof comboMods] }}
                    aria-pressed={comboMods[mod.id as keyof typeof comboMods]}
                    class="rounded border px-3 py-1.5 text-xs font-bold transition {comboMods[mod.id as keyof typeof comboMods] ? 'border-[#6277ff] bg-[#4657c9] text-white' : 'border-edge bg-card text-ink hover:border-[#6277ff]'}"
                  >
                    {mod.label}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Target -->
            <div class="rounded border border-edge bg-canvas/60 p-3">
              <h5 class="mb-2 text-[11px] font-bold text-ink-dim">Target Key</h5>
              <select
                value={comboTarget}
                onchange={(e) => comboTarget = e.currentTarget.value}
                class="w-full rounded-lg border border-edge bg-card px-2 py-2 text-xs font-bold text-ink focus:border-glorious focus:outline-none"
              >
                {#each ['S','T','A','N','D','X','C','V','F','Z'] as k}
                  <option value={k}>{k}</option>
                {/each}
              </select>
            </div>
          </div>

          <!-- Preview -->
          <div class="flex items-center gap-3 rounded border border-[#6277ff]/40 bg-[#4657c9]/10 px-4 py-3">
            <span class="text-xs font-black tracking-widest text-[#aeb8ff]">PREVIEW</span>
            <span class="rounded-full bg-canvas px-3 py-1 text-sm font-extrabold text-ink shadow-sm">{previewCombo()}</span>
          </div>

          <button
            type="button"
            onclick={assignCombination}
            disabled={isLocked || !selectedPin}
            class="rounded bg-glorious px-5 py-2 text-xs font-extrabold text-canvas hover:bg-glorious-soft transition disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Assign Combination
          </button>
        </div>
      {:else if activeCategory === 'shortcuts'}
        <div class="mb-3 flex items-center gap-2">
          <span class="text-[11px] font-bold text-ink-faint">Filter:</span>
          {#each [
            { id: 'all', label: 'All' },
            { id: 'win', label: 'Windows' },
            { id: 'mac', label: 'macOS' },
          ] as f}
            <button
              type="button"
              onclick={() => shortcutFilter = f.id as typeof shortcutFilter}
              aria-pressed={shortcutFilter === f.id}
              class="rounded px-2.5 py-0.5 text-[11px] font-bold transition {shortcutFilter === f.id ? 'border border-[#6277ff] bg-[#4657c9] text-white' : 'border border-edge bg-card text-ink-dim hover:text-ink'}"
            >
              {f.label}
            </button>
          {/each}
        </div>
        <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-1.5 max-h-[135px] overflow-y-auto pr-1">
          {#each filteredShortcuts() as action (action.id)}
            <button
              type="button"
              class="flex items-center justify-center rounded border border-edge bg-canvas/60 px-2.5 py-2 text-center text-xs font-medium text-ink transition hover:border-[#6277ff] hover:bg-[#4657c9]/15 hover:text-white disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!selectedPin || isLocked}
              onclick={() => onAssign('shortcuts', action.label)}
              title={selectedPin ? `Assign ${action.label} to ${currentLabel}` : i18n.t('dock.select_prompt')}
            >
              <span class="truncate">{action.label}</span>
            </button>
          {/each}
        </div>
      {:else if activeCategory === 'macro'}
        <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-1.5 max-h-[135px] overflow-y-auto pr-1">
          {#each macroList as macro}
            <button
              type="button"
              class="flex items-center justify-center rounded border border-edge bg-canvas/60 px-2.5 py-2 text-center text-xs font-medium text-ink transition hover:border-[#6277ff] hover:bg-[#4657c9]/15 hover:text-white disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!selectedPin || isLocked}
              onclick={() => onAssign('macro', `macro:m${macro.slot}`)}
              title={selectedPin ? `Assign ${macro.name} to ${currentLabel}` : i18n.t('dock.select_prompt')}
            >
              <span class="truncate">{macro.name || `M${macro.slot}`}</span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-1.5 max-h-[135px] overflow-y-auto pr-1">
          {#each categoryActions as action (action.id)}
            {@const localizedLabel = getActionLabel(action)}
            <button
              type="button"
              class="flex items-center justify-center rounded border border-edge bg-canvas/60 px-2.5 py-2 text-center text-xs font-medium text-ink transition hover:border-[#6277ff] hover:bg-[#4657c9]/15 hover:text-white disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!selectedPin || isLocked}
              onclick={() => onAssign(activeCategory, action.label)}
              title={selectedPin ? `Assign ${localizedLabel} to ${currentLabel}` : i18n.t('dock.select_prompt')}
            >
              <span class="truncate">{localizedLabel}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

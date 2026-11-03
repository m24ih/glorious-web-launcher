<script lang="ts">
  import type { DeviceCapabilities, PinId, ActionCategory, PinAction } from '../lib/types';
  import { DOCK_CATEGORIES, DOCK_ACTIONS, PIN_LABELS, loadMacros } from '../lib/actions';

  interface Props {
    selectedPin: PinId | null;
    mappings: PinAction[];
    onAssign: (category: ActionCategory, action: string) => void;
    leftClickLocked?: boolean;
    caps?: DeviceCapabilities;
  }

  let { selectedPin, mappings, onAssign, leftClickLocked = false, caps }: Props = $props();

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

  const currentLabel = $derived(
    selectedPin ? PIN_LABELS[selectedPin] : null
  );

  // Load macros when macro category selected to allow direct selection
  let macroList = $derived(activeCategory === 'macro' ? loadMacros() : []);

  const isLocked = $derived(selectedPin === 'left' && leftClickLocked);

  function applyTurbo() {
    const action = turboKey ? `gaming:turbo_${turboKey}` : 'gaming:turbo';
    onAssign('gaming', action);
  }

  function assignCombination() {
    const mods: string[] = [];
    if (comboMods.ctrl) mods.push('ctrl');
    if (comboMods.shift) mods.push('shift');
    if (comboMods.alt) mods.push('alt');
    if (comboMods.win) mods.push('win');
    const combo = mods.length ? `${mods.join('+')}+${comboTarget}` : comboTarget;
    onAssign('combinations', `combo:${combo.toLowerCase()}`);
  }

  const previewCombo = $derived(() => {
    const mods: string[] = [];
    if (comboMods.ctrl) mods.push('Ctrl');
    if (comboMods.shift) mods.push('Shift');
    if (comboMods.alt) mods.push('Alt');
    if (comboMods.win) mods.push('Win / Cmd');
    return mods.length ? `${mods.join(' + ')} + ${comboTarget}` : comboTarget;
  });

  function filteredShortcuts() {
    const all = DOCK_ACTIONS['shortcuts'] ?? [];
    if (shortcutFilter === 'win') {
      return all.filter((a) => /win|ctrl/i.test(a.label) && !/cmd/i.test(a.label));
    }
    if (shortcutFilter === 'mac') {
      return all.filter((a) => /cmd/i.test(a.label) || (/ctrl/i.test(a.label) && !/win/i.test(a.label)));
    }
    return all;
  }
</script>

<div class="border-t border-edge bg-panel px-6 py-4">
  <div class="mx-auto max-w-[1100px] flex flex-col gap-3">
    <!-- Header row: category bar & active button capsule -->
    <div class="flex flex-wrap items-center justify-between gap-3">
      <!-- Category bar with dark zinc + amber highlight -->
      <div class="flex items-center gap-1.5 overflow-x-auto rounded-full border border-edge bg-card p-1">
        {#each DOCK_CATEGORIES as cat (cat.id)}
          <button
            type="button"
            class="rounded-full px-3.5 py-1 text-xs font-semibold transition {activeCategory === cat.id
              ? 'border border-glorious/50 bg-zinc-800 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.18)]'
              : 'border border-transparent text-ink-dim hover:bg-card-hover hover:text-ink'}"
            onclick={() => (activeCategory = cat.id)}
          >
            {cat.label}
          </button>
        {/each}
      </div>

      <!-- Button Rebind Capsule -->
      {#if selectedPin && currentLabel}
        <div class="flex items-center gap-2 rounded-full border border-glorious/40 bg-glorious/10 px-3.5 py-1 text-xs font-medium text-glorious">
          <span class="h-1.5 w-1.5 rounded-full bg-glorious shadow-[0_0_6px_#ffb800]"></span>
          <span>Editing: <strong class="text-ink">{currentLabel}</strong> &rarr; <span class="font-bold">{currentMapping?.action ?? currentLabel}</span></span>
        </div>
      {:else}
        <div class="flex items-center gap-2 rounded-full border border-edge bg-card px-3.5 py-1 text-xs text-ink-faint">
          <span>Select a button pin on the mouse above to rebind</span>
        </div>
      {/if}
    </div>

    <!-- Lock banner for Left Click -->
    {#if isLocked}
      <div class="rounded-xl border border-amber-500/40 bg-amber-950/60 px-4 py-3 text-sm font-medium text-amber-300 shadow-[0_0_16px_rgba(245,158,11,0.15)]">
        <span class="text-lg">🔒</span>
        <span class="ml-2">Left Click is locked for safety. You can disable Left Click Lock in the Advanced tab if you really want to rebind it.</span>
      </div>
    {/if}

    <!-- Action pills grid -->
    <div class="rounded-2xl border border-edge bg-card p-3">
      <!-- Gaming Turbo panel -->
      {#if activeCategory === 'gaming'}
        <div class="space-y-4">
          <div class="flex items-center gap-3">
            <h4 class="text-xs font-black tracking-[0.18em] text-glorious">GAMING TURBO / RAPID FIRE</h4>
            <span class="rounded-full bg-glorious/10 px-2 py-0.5 text-[10px] font-bold text-glorious border border-glorious/30">LIVE</span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- Count -->
            <div class="rounded-xl border border-edge bg-canvas/60 p-3">
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
            <div class="rounded-xl border border-edge bg-canvas/60 p-3">
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
            <div class="rounded-xl border border-edge bg-canvas/60 p-3">
              <span class="mb-1 block text-[11px] font-bold text-ink-dim">Target Key</span>
              <div class="flex gap-1.5">
                {#each ['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z'] as k}
                  <button
                    type="button"
                    onclick={() => turboKey = k}
                    class="rounded-md border px-2 py-1 text-xs font-bold transition {turboKey === k ? 'border-glorious bg-glorious text-canvas shadow-[0_0_8px_rgba(255,184,0,0.35)]' : 'border-edge bg-card text-ink hover:border-glorious hover:text-glorious'}"
                  >
                    {k}
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <!-- Test input -->
          <div class="rounded-xl border border-edge bg-canvas/60 p-3">
            <label for="turbo-test" class="mb-1 block text-[11px] font-bold text-ink-dim">Test Keys — type to feel the repeat</label>
            <input
              id="turbo-test"
              type="text"
              value={testInput}
              oninput={(e) => testInput = e.currentTarget.value}
              placeholder="Type here..."
              class="w-full rounded-lg border border-edge bg-card px-3 py-2 text-sm font-medium text-ink focus:border-glorious focus:outline-none"
            />
          </div>

          {#if !caps?.hasGamingTurbo}
            <div class="rounded-xl border border-amber-500/40 bg-amber-950/40 px-3 py-2 text-[11px] leading-snug text-amber-300">
              Note: Model O Wireless MCU does not store rapid fire on-board; this is active only while software launcher runs.
            </div>
          {/if}

          <button
            type="button"
            onclick={applyTurbo}
            disabled={isLocked || !selectedPin}
            class="rounded-full bg-glorious px-6 py-2 text-xs font-extrabold text-canvas shadow-[0_0_14px_rgba(255,184,0,0.35)] hover:bg-glorious-soft transition disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Apply Turbo
          </button>
        </div>
      {:else if activeCategory === 'combinations'}
        <div class="space-y-4">
          <div class="flex items-center gap-3">
            <h4 class="text-xs font-black tracking-[0.18em] text-glorious">COMBINATION BUILDER</h4>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Modifiers -->
            <div class="rounded-xl border border-edge bg-canvas/60 p-3">
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
                    class="rounded-lg border px-3 py-1.5 text-xs font-bold transition {comboMods[mod.id as keyof typeof comboMods] ? 'border-glorious bg-glorious text-canvas shadow-[0_0_8px_rgba(255,184,0,0.35)]' : 'border-edge bg-card text-ink hover:border-glorious hover:text-glorious'}"
                  >
                    {mod.label}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Target -->
            <div class="rounded-xl border border-edge bg-canvas/60 p-3">
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
          <div class="flex items-center gap-3 rounded-xl border border-glorious/30 bg-glorious/10 px-4 py-3">
            <span class="text-xs font-black tracking-widest text-glorious">PREVIEW</span>
            <span class="rounded-full bg-canvas px-3 py-1 text-sm font-extrabold text-ink shadow-sm">{previewCombo()}</span>
          </div>

          <button
            type="button"
            onclick={assignCombination}
            disabled={isLocked || !selectedPin}
            class="rounded-full bg-glorious px-6 py-2 text-xs font-extrabold text-canvas shadow-[0_0_14px_rgba(255,184,0,0.35)] hover:bg-glorious-soft transition disabled:opacity-40 disabled:cursor-not-allowed"
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
              class="rounded-full px-2.5 py-0.5 text-[11px] font-bold transition {shortcutFilter === f.id ? 'border border-glorious bg-zinc-900 text-glorious shadow-[0_0_8px_rgba(255,184,0,0.25)]' : 'border border-edge bg-card text-ink-dim hover:text-ink'}"
            >
              {f.label}
            </button>
          {/each}
        </div>
        <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-1.5 max-h-[135px] overflow-y-auto pr-1">
          {#each filteredShortcuts() as action (action.id)}
            <button
              type="button"
              class="flex items-center justify-center rounded-xl border border-edge bg-canvas/60 px-2.5 py-2 text-center text-xs font-medium text-ink transition hover:border-glorious hover:bg-glorious/10 hover:text-glorious disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!selectedPin || isLocked}
              onclick={() => onAssign('shortcuts', action.label)}
              title={selectedPin ? `Assign ${action.label} to ${currentLabel}` : 'Select a pin first'}
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
              class="flex items-center justify-center rounded-xl border border-edge bg-canvas/60 px-2.5 py-2 text-center text-xs font-medium text-ink transition hover:border-glorious hover:bg-glorious/10 hover:text-glorious disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!selectedPin || isLocked}
              onclick={() => onAssign('macro', `macro:m${macro.slot}`)}
              title={selectedPin ? `Assign ${macro.name} to ${currentLabel}` : 'Select a pin first'}
            >
              <span class="truncate">{macro.name || `M${macro.slot}`}</span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-1.5 max-h-[135px] overflow-y-auto pr-1">
          {#each categoryActions as action (action.id)}
            <button
              type="button"
              class="flex items-center justify-center rounded-xl border border-edge bg-canvas/60 px-2.5 py-2 text-center text-xs font-medium text-ink transition hover:border-glorious hover:bg-glorious/10 hover:text-glorious disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!selectedPin || isLocked}
              onclick={() => onAssign(activeCategory, action.label)}
              title={selectedPin ? `Assign ${action.label} to ${currentLabel}` : 'Select a pin first'}
            >
              <span class="truncate">{action.label}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

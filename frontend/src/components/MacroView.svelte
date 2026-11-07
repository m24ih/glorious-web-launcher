<script lang="ts">
  import {
    MousePointerClick,
    Move,
    Palette,
    Cpu,
    Trash2,
    Plus,
    Loader2,
    ShieldCheck,
    Copy,
    Download,
    MoveUp,
    MoveDown,
    Search,
    Mic,
    Headphones,
    Key,
    Settings,
    Zap,
    LogOut,
    Info,
    X,
    ChevronDown,
    Save,
    RotateCcw,
  } from 'lucide-svelte';
  import { onDestroy } from 'svelte';
  import {
    loadMacros,
    saveMacros,
    getMacroActionLabel,
    macroDefinitionToRawBytes,
  } from '../lib/actions';
  import { applySettings } from '../lib/api';
  import type {
    DeviceView,
    MacroDefinition,
    MacroEvent,
    MacroItem,
    MacroRepeatMode,
  } from '../lib/types';

  interface Props {
    activeTab: string;
    onTabChange: (tab: string) => void;
    device?: DeviceView | null;
  }

  let { activeTab, onTabChange, device }: Props = $props();

  // State: loaded macros from storage + dirty edits
  let macros = $state<MacroDefinition[]>(loadMacros());
  let editingId = $state<string | null>(null); // which macro we're editing
  let isRecording = $state<boolean>(false);
  let isSyncing = $state<boolean>(false);
  let recordedEvents: MacroEvent[] = $state<MacroEvent[]>([]);
  let recordStart = $state<number | null>(null);

  // Timeline / action blocks
  let timeline = $state<MacroEvent[]>([]);

  // UI state
  let newMacroName = $state<string>('');
  let newMacroSlot = $state<number>(1);
  let repeatMode = $state<MacroRepeatMode>('no_repeat');
  let loopCount = $state<number>(1);
  let searchQuery = $state<string>('');

  // Time Delay Mode: 'disable' | 'dynamic' | 'fixed'
  let timeDelayMode = $state<'disable' | 'dynamic' | 'fixed'>('dynamic');
  let fixedDelayMs = $state<number>(50);

  // Loop settings
  let stopOnAnyKey = $state<boolean>(false);

  // Derived: filtered/visible macros
  let visibleMacros = $derived(
    macros.filter((m) => {
      if (!searchQuery) return true;
      return (
        m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        m.id.toString().includes(searchQuery)
      );
    })
  );

  // Compute badge label from repeat mode
  function getRepeatBadge(mode: string): string {
    switch (mode) {
      case 'no_repeat':
        return '1x';
      case 'toggle':
        return 'Toggle';
      case 'while_holding':
        return 'Hold';
      default:
        return '1x';
    }
  }

  // Record a keydown/keyup event from the window
  function startRecording() {
    isRecording = true;
    recordedEvents = [];
    recordStart = Date.now();

    function handleKeyDown(e: KeyboardEvent) {
      if (!isRecording) return;
      const delta = recordStart ? Date.now() - recordStart : 0;
      recordStart = Date.now();
      recordedEvents.push({
        id: Math.random().toString(36).substring(2),
        type: 'down',
        key: e.key,
        keyCode: e.keyCode,
        delay: delta,
      });
    }

    function handleKeyUp(e: KeyboardEvent) {
      if (!isRecording) return;
      const delta = Date.now() - (recordStart || Date.now());
      recordedEvents.push({
        id: Math.random().toString(36).substring(2),
        type: 'up',
        key: e.key,
        keyCode: e.keyCode,
        delay: delta,
      });
    }

    function handleKeyPress(e: KeyboardEvent) {
      // Prevent default for common keys during recording
      if (/^(Control|Shift|Alt|Meta|Arrow)/i.test(e.key)) {
        e.preventDefault();
      }
    }

    addEventListener('keydown', handleKeyDown);
    addEventListener('keyup', handleKeyUp);
    addEventListener('keypress', handleKeyPress);
  }

  function stopRecording() {
    isRecording = false;
    // Merge recorded events into timeline with proper delay mode handling
    if (recordedEvents.length > 0) {
      const eventsToAdd = recordedEvents.map((ev, idx) => ({
        ...ev,
        // Apply delay mode
        delay: timeDelayMode === 'fixed' ? fixedDelayMs : ev.delay,
      }));
      timeline = [...timeline, ...eventsToAdd];
      recordedEvents = [];
      showToast(`Recorded ${eventsToAdd.length} events`);
    }
  }

  // Add a new empty macro
  function addMacro() {
    const nextSlot = Math.max(1, ...macros.map((m) => m.slot)) + 1;
    const newMacro: MacroDefinition = {
      id: `m${nextSlot}`,
      slot: nextSlot,
      name: `Macro ${nextSlot}`,
      repeatMode: 'no_repeat',
      loopCount: 1,
      events: [],
    };
    macros = [...macros, newMacro];
    editingId = newMacro.id;
    timeline = [];
    newMacroName = newMacro.name;
    newMacroSlot = newMacro.slot;
    repeatMode = newMacro.repeatMode;
    loopCount = newMacro.loopCount;
    timeDelayMode = 'dynamic';
    fixedDelayMs = 50;
    stopOnAnyKey = false;
    saveMacros(macros);
    showToast('New macro created');
  }

  // Delete a macro
  function deleteMacro(id: string) {
    macros = macros.filter((m) => m.id !== id);
    if (editingId === id) {
      editingId = null;
      timeline = [];
      newMacroName = '';
      newMacroSlot = 1;
      repeatMode = 'no_repeat';
      loopCount = 1;
      timeDelayMode = 'dynamic';
      fixedDelayMs = 50;
      stopOnAnyKey = false;
    }
    saveMacros(macros);
    showToast('Macro deleted');
  }

  // Duplicate a macro
  function duplicateMacro(id: string) {
    const original = macros.find((m) => m.id === id);
    if (!original) return;
    const duplicated: MacroDefinition = {
      id: `m${original.slot + 10}`, // bump slot to avoid clash
      slot: original.slot + 10,
      name: `${original.name} Copy`,
      repeatMode: original.repeatMode,
      loopCount: original.loopCount,
      events: [...original.events],
    };
    macros = [...macros, duplicated];
    saveMacros(macros);
    showToast('Macro duplicated');
  }

  // Import macros from JSON
  function importMacros() {
    const json = window.prompt('Paste JSON to import:');
    if (json) {
      try {
        const imported: MacroDefinition[] = JSON.parse(json);
        macros = [...macros, ...imported];
        saveMacros(macros);
        showToast('Macros imported');
      } catch (e) {
        showToast('Invalid JSON format');
      }
    }
  }

  // Export macros as JSON
  function exportMacros() {
    window.navigator.clipboard.writeText(JSON.stringify(macros, null, 2)).then(
      () => showToast('Macros exported to clipboard'),
      () => showToast('Failed to export')
    );
  }

  // Select a macro for editing
  function selectMacro(id: string) {
    const macro = macros.find((m) => m.id === id);
    if (!macro) return;
    editingId = macro.id;
    timeline = [...macro.events];
    newMacroName = macro.name;
    newMacroSlot = macro.slot;
    repeatMode = macro.repeatMode;
    loopCount = macro.loopCount;
    showToast(`Editing ${macro.name}`);
  }

  // Clear all events
  function clearEvents() {
    timeline = [];
    showToast('All events cleared');
  }

  // Move event up
  function moveEventUp(index: number) {
    if (index <= 0) return;
    const newTimeline = [...timeline];
    [newTimeline[index], newTimeline[index - 1]] = [newTimeline[index - 1], newTimeline[index]];
    timeline = newTimeline;
  }

  // Move event down
  function moveEventDown(index: number) {
    if (index >= timeline.length - 1) return;
    const newTimeline = [...timeline];
    [newTimeline[index], newTimeline[index + 1]] = [newTimeline[index + 1], newTimeline[index]];
    timeline = newTimeline;
  }

  // Delete event at index
  function deleteEvent(index: number) {
    const newTimeline = [...timeline];
    newTimeline.splice(index, 1);
    timeline = newTimeline;
  }

  // Save macro
  function saveMacro() {
    if (!editingId) return;
    const macroIndex = macros.findIndex((m) => m.id === editingId);
    if (macroIndex === -1) return;

    const updated: MacroDefinition = {
      ...macros[macroIndex],
      name: newMacroName || `Macro ${newMacroSlot}`,
      repeatMode,
      loopCount,
      events: [...timeline],
    };

    macros = [...macros.slice(0, macroIndex), updated, ...macros.slice(macroIndex + 1)];
    editingId = updated.id;
    newMacroName = updated.name;
    newMacroSlot = updated.slot;
    timeline = [...updated.events];
    repeatMode = updated.repeatMode;
    loopCount = updated.loopCount;
    saveMacros(macros);
    showToast('Macro saved');
  }

  // Sync macros to hardware
  async function syncMacrosToMouse() {
    const targetSlug = device?.slug;
    if (!targetSlug) {
      showToast('No device connected to sync macros to.');
      return;
    }

    try {
      isSyncing = true;
      const macroItems: MacroItem[] = macros.map((m) => {
        const repeat_mode =
          m.repeatMode === 'toggle' ? 1 : m.repeatMode === 'while_holding' ? 2 : 0;
        const loop_count = m.loopCount || 1;
        return {
          id: m.slot,
          name: m.name || `Macro ${m.slot}`,
          repeat_mode,
          loop_count,
          raw_bytes: macroDefinitionToRawBytes(m),
        };
      });

      await applySettings(targetSlug, { macros: macroItems });
      showToast('Macros synced to mouse hardware!');
    } catch (e: any) {
      console.error('Failed to sync macros:', e);
      showToast(e?.message ?? 'Failed to sync macros to mouse');
    } finally {
      isSyncing = false;
    }
  }

  // Get label for display
  function getActionLabel(actionStr: string): string {
    return getMacroActionLabel(actionStr, macros);
  }

  // Toast helper
  let toastMessage = $state<string | null>(null);
  function showToast(msg: string) {
    toastMessage = msg;
    setTimeout(() => (toastMessage = null), 3000);
  }

  // Quick insert actions
  function insertKeyDown() {
    timeline = [...timeline, { id: crypto.randomUUID(), type: 'down', key: '', keyCode: undefined, delay: timeDelayMode === 'fixed' ? fixedDelayMs : 0 }];
  }
  function insertKeyUp() {
    timeline = [...timeline, { id: crypto.randomUUID(), type: 'up', key: '', keyCode: undefined, delay: timeDelayMode === 'fixed' ? fixedDelayMs : 0 }];
  }
  function insertDelay() {
    timeline = [...timeline, { id: crypto.randomUUID(), type: 'delay', delay: 50 }];
  }
  function insertText() {
    timeline = [...timeline, { id: crypto.randomUUID(), type: 'text', text: '', delay: timeDelayMode === 'fixed' ? fixedDelayMs : 0 }];
  }
  function insertMouseClick() {
    timeline = [
      ...timeline,
      {
        id: crypto.randomUUID(),
        type: 'mouse',
        key: 'Left Click',
        mouseButton: 1,
        delay: timeDelayMode === 'fixed' ? fixedDelayMs : 0,
      },
    ];
  }

  // State sync handled directly via selectMacro

  // Cleanup listeners on destroy
  onDestroy(() => {
    stopRecording();
  });
</script>

<div class="min-h-screen bg-canvas text-ink">
  <div class="border-b border-edge/60 bg-panel/30">
    <div class="mx-auto max-w-[1400px] px-6 lg:px-8 py-3 flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3">
      <div class="flex items-start gap-3 rounded-xl border border-glorious/30 bg-amber-950/30 p-3 flex-1">
        <Info class="mt-0.5 flex-shrink-0 h-5 w-5 text-glorious" aria-hidden="true" />
        <div class="flex-1 text-sm text-amber-200 leading-relaxed">
          <span class="font-semibold text-glorious">Macro Assignment:</span>{' '}
          To assign a macro, select or save it here, then head over to the <span class="font-medium underline">Customize</span> tab and click any mouse button.
        </div>
      </div>
      <button
        type="button"
        class="flex items-center justify-center gap-2 rounded-xl bg-glorious px-5 py-3 text-sm font-black text-zinc-950 shadow-[0_0_20px_rgba(255,184,0,0.4)] transition hover:bg-glorious-soft hover:shadow-[0_0_26px_rgba(255,184,0,0.6)] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed shrink-0"
        onclick={syncMacrosToMouse}
        disabled={isSyncing}
        title="Sync and write all macros to mouse onboard memory"
      >
        {#if isSyncing}
          <Loader2 size={16} class="animate-spin" />
          <span>Syncing to Mouse...</span>
        {:else}
          <Zap size={16} class="fill-current" />
          <span>Sync Macros to Mouse</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Main Content Split: Left Sidebar + Right Workspace -->
  <div class="mx-auto max-w-[1400px] flex h-[calc(100vh-120px)] overflow-hidden px-6 lg:px-8">
    <!-- Left Sidebar: Macro List -->
    <aside class="w-80 shrink-0 bg-panel border-r border-edge flex flex-col">
      <!-- Sidebar Header -->
      <div class="border-b border-edge p-4">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-sm font-bold tracking-[0.16em] uppercase text-ink-faint">MACRO LIST</h2>
          <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious tabular-nums">
            {visibleMacros.length} / {macros.length}
          </span>
        </div>
        <button
          class="w-full flex items-center justify-center gap-2 rounded-xl border border-glorious/40 bg-zinc-900 px-3 py-2 text-sm font-bold text-glorious transition hover:border-glorious hover:bg-glorious/10 shadow-[0_0_10px_rgba(255,184,0,0.15)]"
          onclick={addMacro}
        >
          <Plus size={14} />
          <span>+ New Macro</span>
        </button>
      </div>

      <!-- Search Input -->
      <div class="p-4 border-b border-edge">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-ink-faint" aria-hidden="true" />
          <input
            type="text"
            placeholder="Search macros..."
            class="w-full rounded-xl border border-edge bg-canvas pl-10 pr-4 py-2 text-sm outline-none focus:border-glorious focus:ring-1 focus:ring-glorious/30 placeholder:text-ink-faint"
            oninput={(e: Event) => (searchQuery = (e.target as HTMLInputElement).value)}
          />
        </div>
      </div>

      <!-- Import / Export Buttons -->
      <div class="p-4 border-b border-edge flex gap-2">
        <button
          class="flex-1 flex items-center justify-center gap-1.5 rounded-xl border border-edge bg-canvas px-3 py-2 text-sm font-medium text-ink-dim transition hover:border-edge-bright hover:text-ink"
          onclick={importMacros}
          title="Import JSON"
        >
          <Download size={14} />
          <span>Import</span>
        </button>
        <button
          class="flex-1 flex items-center justify-center gap-1.5 rounded-xl border border-edge bg-canvas px-3 py-2 text-sm font-medium text-ink-dim transition hover:border-edge-bright hover:text-ink"
          onclick={exportMacros}
          title="Export JSON"
        >
          <Copy size={14} />
          <span>Export</span>
        </button>
      </div>

      <!-- Macro List Items -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2">
        {#if visibleMacros.length === 0}
          <div class="flex flex-col items-center justify-center h-full text-ink-faint py-12">
            <Palette class="h-12 w-12 mb-3 opacity-30" />
            <p class="text-sm">No macros found</p>
            <p class="text-xs mt-1">Click "+ New Macro" to create one</p>
          </div>
        {:else}
          {#each visibleMacros as macro}
            {@const isSelected = editingId === macro.id}
            <div
              role="button"
              tabindex="0"
              class="w-full relative rounded-xl p-3 text-left transition cursor-pointer flex items-start gap-3 group {isSelected
                ? 'border-glorious bg-zinc-900/95 ring-1 ring-glorious/60 shadow-[0_0_18px_rgba(255,184,0,0.20)]'
                : 'border-edge bg-canvas/60 hover:border-edge-bright hover:bg-card-hover'}"
              onclick={() => selectMacro(macro.id)}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectMacro(macro.id); }}
            >
              <!-- Slot Pill -->
              <span class="flex-shrink-0 flex items-center justify-center rounded-full border border-glorious/40 bg-glorious/10 px-2.5 py-1 text-[11px] font-black text-glorious min-w-[3rem]">
                M{macro.slot}
              </span>

              <!-- Macro Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span class="text-sm font-semibold truncate {isSelected ? 'text-ink' : 'text-ink-dim'}">{macro.name}</span>
                  <span class="flex-shrink-0 rounded-full border px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider {macro.repeatMode === 'no_repeat' ? 'border-edge bg-canvas text-ink-dim' : macro.repeatMode === 'toggle' ? 'border-amber-500/40 bg-amber-950/30 text-amber-300' : 'border-emerald-500/40 bg-emerald-950/30 text-emerald-300'}">
                    {getRepeatBadge(macro.repeatMode)}
                  </span>
                </div>
                <div class="flex items-center gap-2 text-[11px] text-ink-faint">
                  <span>{macro.events.length} actions</span>
                  <span class="hidden sm:inline">•</span>
                  <span class="hidden sm:inline">Slot {macro.slot}</span>
                </div>
              </div>

              <!-- Actions: Edit (Select), Duplicate, Delete -->
              <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition">
                <button
                  type="button"
                  class="rounded-lg p-1.5 text-ink-faint hover:text-glorious hover:bg-glorious/10 transition"
                  onclick={(e) => { e.stopPropagation(); selectMacro(macro.id); }}
                  title="Select for editing"
                  aria-label="Select macro"
                >
                  <Key size={14} />
                </button>
                <button
                  type="button"
                  class="rounded-lg p-1.5 text-ink-faint hover:text-yellow-300 hover:bg-yellow-500/10 transition"
                  onclick={(e) => { e.stopPropagation(); duplicateMacro(macro.id); }}
                  title="Duplicate macro"
                  aria-label="Duplicate macro"
                >
                  <RotateCcw size={14} />
                </button>
                <button
                  type="button"
                  class="rounded-lg p-1.5 text-ink-faint hover:text-red-400 hover:bg-red-500/10 transition"
                  onclick={(e) => { e.stopPropagation(); deleteMacro(macro.id); }}
                  title="Delete macro"
                  aria-label="Delete macro"
                >
                  <Trash2 size={14} />
                </button>
              </div>

              <!-- Selected indicator -->
              {#if isSelected}
                <div class="absolute inset-0 pointer-events-none rounded-xl border-2 border-glorious/50" aria-hidden="true"></div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </aside>

    <!-- Right Workspace -->
    <main class="flex-1 overflow-y-auto p-6 lg:p-8 space-y-6">
      {#if !editingId}
        <!-- Empty State -->
        <div class="flex flex-col items-center justify-center h-full text-center">
          <Palette class="h-20 w-20 text-ink-faint/20 mb-6" />
          <h3 class="text-lg font-bold text-ink-dim mb-2">Select or Create a Macro</h3>
          <p class="text-sm text-ink-faint max-w-md">
            Choose a macro from the sidebar to edit its sequence, or click "+ New Macro" to start fresh.
          </p>
          <button
            class="mt-6 flex items-center gap-2 rounded-xl border border-glorious/40 bg-zinc-900 px-5 py-3 text-sm font-bold text-glorious transition hover:border-glorious hover:bg-glorious/10 shadow-[0_0_10px_rgba(255,184,0,0.15)]"
            onclick={addMacro}
          >
            <Plus size={16} />
            <span>Create New Macro</span>
          </button>
        </div>
      {:else}
        <!-- Card 1: MACRO SETTINGS & TIME DELAY -->
        <section class="rounded-2xl border border-edge bg-card p-5">
          <div class="mb-4 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">MACRO SETTINGS & TIME DELAY</span>
              <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious">
                Slot M{newMacroSlot}
              </span>
            </div>
            {#if isRecording}
              <span class="flex items-center gap-1.5 rounded-full border border-red-500/40 bg-red-500/10 px-3 py-1 text-[11px] font-bold text-red-400 animate-pulse">
                <span class="h-2 w-2 rounded-full bg-red-400"></span>
                RECORDING
              </span>
            {/if}
          </div>

          <!-- Macro Name -->
          <div class="mb-5">
            <label for="macro-name-input" class="block text-[11px] font-bold tracking-[0.1em] uppercase text-ink-faint mb-2">Macro Name</label>
            <input
              id="macro-name-input"
              type="text"
              bind:value={newMacroName}
              class="w-full rounded-xl border border-edge bg-canvas px-4 py-2.5 text-sm outline-none focus:border-glorious focus:ring-1 focus:ring-glorious/30 placeholder:text-ink-faint"
              placeholder="Enter macro name..."
            />
          </div>

          <!-- Time Delay Mode (Pills/Radio Tabs) -->
          <div class="mb-5">
            <span class="block text-[11px] font-bold tracking-[0.1em] uppercase text-ink-faint mb-3">Time Delay Mode</span>
            <div class="flex gap-2" role="radiogroup" aria-label="Time delay mode">
              {#each [
                { value: 'disable', label: 'Disable Delay', desc: 'No delays between events' },
                { value: 'dynamic', label: 'Dynamic Delay', desc: 'Recorded timing preserved' },
                { value: 'fixed', label: 'Fixed Delay', desc: 'Custom ms between events' },
              ] as mode}
                <button
                  type="button"
                  role="radio"
                  aria-checked={timeDelayMode === mode.value}
                  class="flex-1 rounded-xl border p-3 text-left transition cursor-pointer {timeDelayMode === mode.value
                    ? 'border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/50 shadow-[0_0_14px_rgba(255,184,0,0.22)]'
                    : 'border border-edge bg-canvas/50 text-ink-dim hover:border-edge-bright hover:text-ink'}"
                  onclick={() => (timeDelayMode = mode.value as 'disable' | 'dynamic' | 'fixed')}
                >
                  <div class="text-sm font-extrabold">{mode.label}</div>
                  <div class="text-[10px] text-ink-faint mt-0.5 font-normal">{mode.desc}</div>
                </button>
              {/each}
            </div>
            {#if timeDelayMode === 'fixed'}
              <div class="mt-3 flex items-center gap-3">
                <label for="fixed-delay-input" class="text-[11px] font-medium text-ink-dim">Fixed Delay (ms):</label>
                <input
                  id="fixed-delay-input"
                  type="number"
                  bind:value={fixedDelayMs}
                  min="0"
                  max="10000"
                  step="10"
                  class="w-24 rounded-xl border border-edge bg-canvas px-3 py-2 text-sm font-mono tabular-nums outline-none focus:border-glorious"
                />
                <span class="text-[11px] text-ink-faint">Applied to all new recorded events</span>
              </div>
            {/if}
          </div>

          <!-- Loop Settings (Keychron Style) -->
          <div class="mb-5 rounded-xl border border-edge/80 bg-panel/70 p-4">
            <div class="mb-3 flex items-center justify-between">
              <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">LOOP SETTINGS</h3>
              <span class="text-[10px] text-ink-faint">How the macro repeats when assigned to a button</span>
            </div>

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
              <!-- Stop on Release (while_holding) -->
              <button
                type="button"
                role="radio"
                aria-checked={repeatMode === 'while_holding'}
                class="rounded-xl border p-3 text-left transition cursor-pointer {repeatMode === 'while_holding'
                  ? 'border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/50 shadow-[0_0_12px_rgba(255,184,0,0.20)]'
                  : 'border border-edge bg-canvas/50 text-ink-dim hover:border-edge-bright hover:text-ink'}"
                onclick={() => (repeatMode = 'while_holding')}
              >
                <div class="flex items-center gap-2 mb-1">
                  <span class="flex-shrink-0 h-5 w-5 rounded border-2 border-current" style="border-color: {repeatMode === 'while_holding' ? '#ffb800' : 'var(--color-edge)'};">
                    {#if repeatMode === 'while_holding'}
                      <span class="flex h-full w-full items-center justify-center">
                        <span class="h-2.5 w-2.5 rounded bg-glorious"></span>
                      </span>
                    {/if}
                  </span>
                  <span class="text-sm font-bold">Stop on Release</span>
                </div>
                <div class="text-[10px] text-ink-faint leading-snug pl-7">
                  Repeats while button is held. Stops immediately on release.
                </div>
              </button>

              <!-- Stop on Any Key (toggle) -->
              <button
                type="button"
                role="radio"
                aria-checked={repeatMode === 'toggle'}
                class="rounded-xl border p-3 text-left transition cursor-pointer {repeatMode === 'toggle'
                  ? 'border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/50 shadow-[0_0_12px_rgba(255,184,0,0.20)]'
                  : 'border border-edge bg-canvas/50 text-ink-dim hover:border-edge-bright hover:text-ink'}"
                onclick={() => (repeatMode = 'toggle')}
              >
                <div class="flex items-center gap-2 mb-1">
                  <span class="flex-shrink-0 h-5 w-5 rounded border-2 border-current" style="border-color: {repeatMode === 'toggle' ? '#ffb800' : 'var(--color-edge)'};">
                    {#if repeatMode === 'toggle'}
                      <span class="flex h-full w-full items-center justify-center">
                        <span class="h-2.5 w-2.5 rounded bg-glorious"></span>
                      </span>
                    {/if}
                  </span>
                  <span class="text-sm font-bold">Stop on Any Key</span>
                </div>
                <div class="text-[10px] text-ink-faint leading-snug pl-7">
                  Toggles on press, stops when any other key is pressed.
                </div>
              </button>

              <!-- Stop on Press (no_repeat with count) -->
              <button
                type="button"
                role="radio"
                aria-checked={repeatMode === 'no_repeat'}
                class="rounded-xl border p-3 text-left transition cursor-pointer {repeatMode === 'no_repeat'
                  ? 'border-glorious/80 bg-zinc-900 text-glorious ring-1 ring-glorious/50 shadow-[0_0_12px_rgba(255,184,0,0.20)]'
                  : 'border border-edge bg-canvas/50 text-ink-dim hover:border-edge-bright hover:text-ink'}"
                onclick={() => (repeatMode = 'no_repeat')}
              >
                <div class="flex items-center gap-2 mb-1">
                  <span class="flex-shrink-0 h-5 w-5 rounded border-2 border-current" style="border-color: {repeatMode === 'no_repeat' ? '#ffb800' : 'var(--color-edge)'};">
                    {#if repeatMode === 'no_repeat'}
                      <span class="flex h-full w-full items-center justify-center">
                        <span class="h-2.5 w-2.5 rounded bg-glorious"></span>
                      </span>
                    {/if}
                  </span>
                  <span class="text-sm font-bold">Stop on Press</span>
                </div>
                <div class="text-[10px] text-ink-faint leading-snug pl-7">
                  Plays once per press. Loop count:{" "}
                  <input
                    type="number"
                    bind:value={loopCount}
                    min="1"
                    max="100"
                    class="w-14 rounded border border-edge bg-canvas px-2 py-1 text-sm font-mono tabular-nums text-center outline-none focus:border-glorious"
                  />
                </div>
              </button>
            </div>

            {#if stopOnAnyKey && repeatMode === 'toggle'}
              <div class="mt-3 flex items-center gap-2 rounded-lg border border-amber-500/40 bg-amber-950/20 px-3 py-2 text-[11px] text-amber-300">
                <ShieldCheck size={13} class="shrink-0" />
                <span>Stop on Any Key is enabled — macro will interrupt on any keystroke.</span>
              </div>
            {/if}
          </div>

          <!-- Action Buttons -->
          <div class="flex flex-wrap gap-3">
            <!-- Recording Toggle Button (Large Tactile) -->
            <button
              type="button"
              class="flex-1 min-w-[200px] flex items-center justify-center gap-2 rounded-xl border p-4 text-left transition cursor-pointer {isRecording
                ? 'border-red-500/60 bg-red-500/10 text-red-300 shadow-[0_0_18px_rgba(239,68,68,0.25)] animate-pulse-soft'
                : 'border-glorious/40 bg-zinc-900 text-glorious hover:border-glorious hover:bg-glorious/10 shadow-[0_0_10px_rgba(255,184,0,0.15)]'}"
              onclick={isRecording ? stopRecording : startRecording}
              aria-pressed={isRecording}
            >
              <div class="flex-shrink-0 flex h-10 w-10 items-center justify-center rounded-lg {isRecording ? 'bg-red-500/20' : 'bg-glorious/20'}">
                {#if isRecording}
                  <span class="h-5 w-5 rounded bg-red-400 animate-pulse shadow-[0_0_8px_#ef4444]"></span>
                {:else}
                  <Mic class="h-5 w-5" />
                {/if}
              </div>
              <div class="text-sm font-bold">
                {isRecording ? 'Stop Recording' : 'Start Recording'}
              </div>
              <div class="text-[10px] font-medium opacity-70">
                {isRecording ? 'Press keys to capture...' : 'Captures keystrokes with timing'}
              </div>
            </button>

            <!-- Save Macro (Gold Button) -->
            <button
              type="button"
              class="flex-1 min-w-[140px] flex items-center justify-center gap-2 rounded-xl bg-glorious px-5 py-3 text-sm font-black text-zinc-950 shadow-[0_0_18px_rgba(255,184,0,0.35)] transition hover:bg-glorious-soft hover:shadow-[0_0_24px_rgba(255,184,0,0.5)] cursor-pointer"
              onclick={saveMacro}
            >
              <Save size={15} />
              <span>Save Macro</span>
            </button>

            <!-- Clear All (Subtle Danger) -->
            <button
              type="button"
              class="flex-1 min-w-[140px] flex items-center justify-center gap-2 rounded-xl border border-red-500/30 bg-red-500/5 px-5 py-3 text-sm font-bold text-red-400 transition hover:border-red-500/60 hover:bg-red-500/10 cursor-pointer"
              onclick={clearEvents}
            >
              <RotateCcw size={15} />
              <span>Clear All</span>
            </button>
          </div>
        </section>

        <!-- Card 2: SEQUENCE / EVENT TABLE -->
        <section class="rounded-2xl border border-edge bg-card p-5">
          <div class="mb-4 flex items-center justify-between">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">SEQUENCE / EVENT TABLE</h3>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-[11px] font-bold text-glorious tabular-nums">
              {timeline.length} Events
            </span>
          </div>

          {#if timeline.length === 0}
            <div class="rounded-xl border-2 border-dashed border-edge/50 bg-canvas/50 p-12 text-center">
              <Move class="h-10 w-10 mx-auto text-ink-faint/30 mb-3" />
              <p class="text-sm text-ink-dim mb-1">No events recorded yet</p>
              <p class="text-xs text-ink-faint">Click "Start Recording" or use Quick Insert below</p>
            </div>
          {:else}
            <div class="overflow-x-auto rounded-xl border border-edge">
              <table class="w-full text-sm">
                <thead>
                  <tr class="border-b border-edge bg-panel/50">
                    <th class="w-12 px-3 py-2 text-left text-[10px] font-bold uppercase tracking-wider text-ink-faint">#</th>
                    <th class="w-28 px-3 py-2 text-left text-[10px] font-bold uppercase tracking-wider text-ink-faint">TYPE</th>
                    <th class="flex-1 px-3 py-2 text-left text-[10px] font-bold uppercase tracking-wider text-ink-faint">KEY / VALUE</th>
                    <th class="w-28 px-3 py-2 text-right text-[10px] font-bold uppercase tracking-wider text-ink-faint">DELAY (ms)</th>
                    <th class="w-32 px-3 py-2 text-center text-[10px] font-bold uppercase tracking-wider text-ink-faint">ACTIONS</th>
                  </tr>
                </thead>
                <tbody>
                  {#each timeline as event, index}
                    <tr class="border-b border-edge/50 last:border-0 hover:bg-card-hover transition">
                      <td class="px-3 py-2.5 text-center text-[11px] font-mono tabular-nums text-ink-faint">{index + 1}</td>
                      
                      <td class="px-3 py-2.5 text-center">
                        {#if event.type === 'down'}
                          <span class="inline-flex items-center justify-center rounded-full border border-cyan-500/40 bg-cyan-500/10 px-2 py-0.5 text-[10px] font-bold text-cyan-400">
                            <span class="flex-shrink-0 h-3 w-3" aria-hidden="true">⇩</span>
                            <span class="ml-1 hidden sm:inline">Key Down</span>
                          </span>
                        {:else if event.type === 'up'}
                          <span class="inline-flex items-center justify-center rounded-full border border-rose-500/40 bg-rose-500/10 px-2 py-0.5 text-[10px] font-bold text-rose-400">
                            <span class="flex-shrink-0 h-3 w-3" aria-hidden="true">⇧</span>
                            <span class="ml-1 hidden sm:inline">Key Up</span>
                          </span>
                        {:else if event.type === 'delay'}
                          <span class="inline-flex items-center justify-center rounded-full border border-amber-500/40 bg-amber-500/10 px-2 py-0.5 text-[10px] font-bold text-amber-400">
                            <span class="flex-shrink-0 h-3 w-3" aria-hidden="true">⏱</span>
                            <span class="ml-1 hidden sm:inline">Delay</span>
                          </span>
                        {:else if event.type === 'text'}
                          <span class="inline-flex items-center justify-center rounded-full border border-purple-500/40 bg-purple-500/10 px-2 py-0.5 text-[10px] font-bold text-purple-400">
                            <span class="flex-shrink-0 h-3 w-3" aria-hidden="true">Aa</span>
                            <span class="ml-1 hidden sm:inline">Text</span>
                          </span>
                        {:else if event.type === 'mouse'}
                          <span class="inline-flex items-center justify-center rounded-full border border-emerald-500/40 bg-emerald-500/10 px-2 py-0.5 text-[10px] font-bold text-emerald-400">
                            <span class="flex-shrink-0 h-3 w-3" aria-hidden="true">🖱</span>
                            <span class="ml-1 hidden sm:inline">Mouse</span>
                          </span>
                        {/if}
                      </td>

                      <td class="px-3 py-2.5">
                        {#if event.type === 'down' || event.type === 'up'}
                          <input
                            type="text"
                            value={event.key || ''}
                            class="w-full rounded border border-edge bg-canvas px-2 py-1.5 text-sm font-mono outline-none focus:border-glorious placeholder:text-ink-faint"
                            placeholder="Key (e.g. Ctrl, A, F1)..."
                            oninput={(e) => (event.key = (e.target as HTMLInputElement).value)}
                          />
                        {:else if event.type === 'delay'}
                          <input
                            type="number"
                            value={event.delay ?? 0}
                            min="0"
                            max="10000"
                            step="10"
                            class="w-full rounded border border-edge bg-canvas px-2 py-1.5 text-sm font-mono tabular-nums text-right outline-none focus:border-glorious"
                            oninput={(e) => (event.delay = parseInt((e.target as HTMLInputElement).value || '0') || 0)}
                          />
                        {:else if event.type === 'text'}
                          <input
                            type="text"
                            value={event.text || ''}
                            class="w-full rounded border border-edge bg-canvas px-2 py-1.5 text-sm outline-none focus:border-glorious placeholder:text-ink-faint"
                            placeholder="Text to type..."
                            oninput={(e) => (event.text = (e.target as HTMLInputElement).value)}
                          />
                        {:else if event.type === 'mouse'}
                          <select
                            value={event.key || 'Left Click'}
                            class="w-full rounded border border-edge bg-canvas px-2 py-1.5 text-sm outline-none focus:border-glorious"
                            onchange={(e) => {
                              const val = (e.target as HTMLSelectElement).value;
                              event.key = val;
                              if (val === 'Left Click') event.mouseButton = 1;
                              else if (val === 'Right Click') event.mouseButton = 2;
                              else if (val === 'Middle Click') event.mouseButton = 4;
                              else if (val === 'Backward') event.mouseButton = 8;
                              else if (val === 'Forward') event.mouseButton = 16;
                            }}
                          >
                            <option value="Left Click">Left Click</option>
                            <option value="Right Click">Right Click</option>
                            <option value="Middle Click">Middle Click</option>
                            <option value="Backward">Backward (Side Back)</option>
                            <option value="Forward">Forward (Side Front)</option>
                          </select>
                        {/if}
                      </td>

                      <td class="px-3 py-2.5 text-right">
                        {#if event.type === 'delay'}
                          <span class="text-[11px] font-mono tabular-nums text-amber-300">{event.delay ?? 0}</span>
                        {:else if event.type === 'text'}
                          <span class="text-[11px] text-ink-faint">—</span>
                        {:else}
                          <input
                            type="number"
                            value={event.delay ?? 0}
                            min="0"
                            max="10000"
                            step="10"
                            class="w-20 rounded border border-edge bg-canvas px-2 py-1.5 text-sm font-mono tabular-nums text-right outline-none focus:border-glorious"
                            oninput={(e) => (event.delay = parseInt((e.target as HTMLInputElement).value || '0') || 0)}
                          />
                        {/if}
                      </td>

                      <td class="px-3 py-2.5 text-center">
                        <div class="flex items-center justify-center gap-1">
                          <button
                            class="rounded-lg p-1.5 text-ink-faint hover:text-glorious hover:bg-glorious/10 transition disabled:opacity-30 disabled:cursor-not-allowed"
                            onclick={() => moveEventUp(index)}
                            disabled={index === 0}
                            title="Move up"
                            aria-label="Move up"
                          >
                            <MoveUp size={13} />
                          </button>
                          <button
                            class="rounded-lg p-1.5 text-ink-faint hover:text-glorious hover:bg-glorious/10 transition disabled:opacity-30 disabled:cursor-not-allowed"
                            onclick={() => moveEventDown(index)}
                            disabled={index === timeline.length - 1}
                            title="Move down"
                            aria-label="Move down"
                          >
                            <MoveDown size={13} />
                          </button>
                          <button
                            class="rounded-lg p-1.5 text-ink-faint hover:text-red-400 hover:bg-red-500/10 transition"
                            onclick={() => deleteEvent(index)}
                            title="Delete"
                            aria-label="Delete event"
                          >
                            <Trash2 size={13} class="text-red-400" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}

          <!-- Quick Insert Bar -->
          <div class="mt-4 rounded-xl border border-edge/80 bg-panel/70 p-4">
            <div class="mb-3 flex items-center justify-between">
              <span class="text-[11px] font-bold tracking-[0.1em] uppercase text-ink-faint">QUICK INSERT</span>
              <span class="text-[10px] text-ink-faint">Appends to end of sequence</span>
            </div>
            <div class="grid grid-cols-2 gap-2 sm:grid-cols-5">
              <button
                type="button"
                class="flex items-center justify-center gap-2 rounded-xl border border-edge bg-canvas px-3 py-2.5 text-sm font-medium text-ink-dim transition hover:border-cyan-500/40 hover:text-cyan-400 hover:bg-cyan-500/5 cursor-pointer"
                onclick={insertKeyDown}
              >
                <span class="h-4 w-4 text-cyan-400">⇩</span>
                <span>Key Down</span>
              </button>
              <button
                type="button"
                class="flex items-center justify-center gap-2 rounded-xl border border-edge bg-canvas px-3 py-2.5 text-sm font-medium text-ink-dim transition hover:border-rose-500/40 hover:text-rose-400 hover:bg-rose-500/5 cursor-pointer"
                onclick={insertKeyUp}
              >
                <span class="h-4 w-4 text-rose-400">⇧</span>
                <span>Key Up</span>
              </button>
              <button
                type="button"
                class="flex items-center justify-center gap-2 rounded-xl border border-edge bg-canvas px-3 py-2.5 text-sm font-medium text-ink-dim transition hover:border-amber-500/40 hover:text-amber-400 hover:bg-amber-500/5 cursor-pointer"
                onclick={insertDelay}
              >
                <span class="h-4 w-4 text-amber-400">⏱</span>
                <span>Delay (50ms)</span>
              </button>
              <button
                type="button"
                class="flex items-center justify-center gap-2 rounded-xl border border-edge bg-canvas px-3 py-2.5 text-sm font-medium text-ink-dim transition hover:border-purple-500/40 hover:text-purple-400 hover:bg-purple-500/5 cursor-pointer"
                onclick={insertText}
              >
                <span class="h-4 w-4 text-purple-400">Aa</span>
                <span>Text Block</span>
              </button>
              <button
                type="button"
                class="flex items-center justify-center gap-2 rounded-xl border border-edge bg-canvas px-3 py-2.5 text-sm font-medium text-ink-dim transition hover:border-emerald-500/40 hover:text-emerald-400 hover:bg-emerald-500/5 cursor-pointer"
                onclick={insertMouseClick}
              >
                <MousePointerClick size={15} class="text-emerald-400" />
                <span>Mouse Click</span>
              </button>
            </div>
          </div>
        </section>

        <!-- Card 3: TEST SCRATCHPAD -->
        <section class="rounded-2xl border border-edge bg-card p-5">
          <div class="mb-3 flex items-center justify-between">
            <h3 class="text-[11px] font-bold tracking-[0.16em] uppercase text-ink-faint">TEST SCRATCHPAD</h3>
            <span class="text-[10px] text-ink-faint">Type to test playback simulation</span>
          </div>
          <textarea
            rows="4"
            placeholder="Click here and test your macro by typing... (simulates key presses)"
            class="w-full rounded-xl border border-edge bg-canvas px-4 py-3 text-sm font-mono outline-none focus:border-glorious focus:ring-1 focus:ring-glorious/30 placeholder:text-ink-faint resize-none"
            oninput={(e) => {
              const value = (e.target as HTMLTextAreaElement).value;
              if (value.length > 0 && value.length % 20 === 0) {
                showToast(`Typed: ${value.substring(0, 30)}...`);
              }
            }}
          ></textarea>
          <p class="mt-2 text-[10px] text-ink-faint">
            This scratchpad captures keystrokes for visual feedback only. Actual macro playback occurs when the macro is assigned to a mouse button in the Customize tab.
          </p>
        </section>
      {/if}
    </main>
  </div>

  <!-- Toast Notification -->
  {#if toastMessage}
    <div
      class="fixed top-4 right-4 z-50 flex items-center gap-2 rounded-full border border-glorious/50 bg-zinc-900/95 px-4 py-2 text-sm font-bold text-glorious shadow-[0_0_24px_rgba(255,184,0,0.4)] backdrop-blur animate-in slide-in-from-top-2 duration-300"
      role="status"
      aria-live="polite"
    >
      <span class="h-2 w-2 rounded-full bg-glorious shadow-[0_0_8px_#ffb800]"></span>
      <span>{toastMessage}</span>
    </div>
  {/if}
</div>
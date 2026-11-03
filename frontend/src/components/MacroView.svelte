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
  } from 'lucide-svelte';
  import { onDestroy } from 'svelte';
  import { loadMacros, saveMacros, getMacroActionLabel } from '../lib/actions';
  import type { MacroDefinition, MacroEvent } from '../lib/types';

  interface Props {
    activeTab: string;
    onTabChange: (tab: string) => void;
  }

  let { activeTab, onTabChange }: Props = $props();

  // State: loaded macros from storage + dirty edits
  let macros = $state<MacroDefinition[]>(loadMacros());
  let editingId: string | null = null; // which macro we're editing
  let isRecording = $state<boolean>(false);
  let recordedEvents: MacroEvent[] = $state<MacroEvent[]>([]);
  let recordStart = $state<number | null>(null);

  // Timeline / action blocks
  let timeline = $state<MacroEvent[]>([]);

  // UI state
  let newMacroName = $state<string>('');
  let newMacroSlot = $state<number>(1);
  let repeatMode = $state<'no_repeat' | 'toggle' | 'while_holding'>('no_repeat');
  let loopCount = $state<number>(1);
  let searchQuery = $state<string>('');

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
    saveMacros(macros);
  }

  // Delete a macro
  function deleteMacro(id: string) {
    macros = macros.filter((m) => m.id !== id);
    if (editingId === id) {
      editingId = null;
      timeline = [];
    }
    saveMacros(macros);
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
  }

  // Import macros from JSON
  function importMacros() {
    // In a real app we'd use an <input type="file"> but for now simulate
    // with a prompt. In production this would be triggered by the Export/Import buttons.
    const json = window.prompt('Paste JSON to import:');
    if (json) {
      try {
        const imported: MacroDefinition[] = JSON.parse(json);
        macros = [...macros, ...imported];
        saveMacros(macros);
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
    editingId = null;
    timeline = [];
    saveMacros(macros);
    showToast('Macro saved');
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

  // State sync handled directly via selectMacro

  // Cleanup listeners on destroy
  onDestroy(() => {
    stopRecording();
  });
</script>

<div class="min-h-screen bg-canvas text-ink p-6">

  <!-- Left Sidebar: Macro List -->
  <aside class="w-64 shrink-0 bg-panel border-r border-edge p-4 space-y-4">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-bold">Macro Builder</h2>
      <button
        class="rounded border px-2 py-1 text-sm text-ink-dim hover:text-ink"
        onclick={importMacros}
        title="Import JSON"
      >
        <Search size={14} /> Import
      </button>
      <button
        class="rounded border px-2 py-1 text-sm text-ink-dim hover:text-ink"
        onclick={exportMacros}
        title="Export JSON"
      >
        <Download size={14} /> Export
      </button>
    </div>

    <div>
      <button
        class="w-full flex items-center justify-between rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 hover:text-glorious mb-2"
        onclick={addMacro}
      >
        <Plus size={14} /> + Add Macro
      </button>

      <!-- Search -->
      <div class="mb-3">
        <input
          type="text"
          placeholder="Search macros..."
          class="w-full rounded border px-3 py-2 text-sm background-card outline-none"
          oninput={(e: Event) => (searchQuery = (e.target as HTMLInputElement).value)}
        />
      </div>

      {#each visibleMacros as macro}
      <div
        class="rounded border px-3 py-2 text-sm flex items-center justify-between border-edge background-card mb-2"
      >
        <div class="flex items-center gap-2">
          <span class="text-[14px] font-medium">{`M${macro.slot}`}</span>
          <span class="text-xs font-semibold text-glorious capitalize">{getRepeatBadge(
            macro.repeatMode
          )}</span>
        </div>
        <span class="text-[11px] text-ink-faint">{macro.events.length} actions</span>
        <button
          class="rounded border px-2 py-1 text-[10px] text-ink-dim hover:text-glorious transition"
          onclick={() => selectMacro(macro.id)}
          title="Edit macro"
        >
          ✎
        </button>
        <button
          class="rounded border px-2 py-1 text-[10px] text-ink-dim hover:text-red-400 transition"
          onclick={() => deleteMacro(macro.id)}
          title="Delete macro"
        >
          <Trash2 size={12} class="text-red-400" /> Delete
        </button>
        <button
          class="rounded border px-2 py-1 text-[10px] text-ink-dim hover:text-yellow-300 transition"
          onclick={() => duplicateMacro(macro.id)}
          title="Duplicate macro"
        >
          <Plus size={12} class="text-yellow-300" /> Dup
        </button>
      </div>
      {/each}
    </div>
  </aside>

  <!-- Main Area: Macro Editor -->
  <main class="flex-1 flex flex-col space-y-4 p-6">
    <!-- Header: Name + Recording + Repeat Mode -->
    <div class="rounded border p-4 bg-panel border-edge">
      <div class="flex items-center gap-3 mb-4">
        <input
          type="text"
          bind:value={newMacroName}
          class="flex-1 rounded border px-3 py-2 text-sm outline-none background-card"
          placeholder="Macro name"
        />
        <button
          class="rounded border px-3 py-2 text-sm font-medium text-glorious bg-glorious/10 hover:bg-glorious transition"
          onclick={saveMacro}
          title="Save macro"
        >
          Save
        </button>
      </div>

      <!-- Recording control -->
      <div class="flex items-center gap-2 mb-4">
        <button
          class="flex-1 rounded-full p-2 {isRecording ? 'bg-red-500' : 'bg-glorious/10'} text-{isRecording ? 'red-400' : 'glorious'}"
          onclick={isRecording ? stopRecording : startRecording}
          title={isRecording ? 'Stop Recording' : 'Record Keystrokes'}
        >
          {isRecording ? '■' : '●'}
        </button>
        <span class="text-sm text-ink-faint">
          {isRecording ? 'Recording…' : 'Click keys to record'}
        </span>
      </div>

      <!-- Repeat Mode selector -->
      <div class="flex gap-2 mb-4">
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 {repeatMode === 'no_repeat' ? 'bg-glorious/10 text-glorious' : 'text-ink-dim'}"
          onclick={() => (repeatMode = 'no_repeat')}
        >
          No Repeat
        </button>
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 {repeatMode === 'toggle' ? 'bg-glorious/10 text-glorious' : 'text-ink-dim'}"
          onclick={() => (repeatMode = 'toggle')}
        >
          Toggle
        </button>
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 {repeatMode === 'while_holding' ? 'bg-glorious/10 text-glorious' : 'text-ink-dim'}"
          onclick={() => (repeatMode = 'while_holding')}
        >
          Hold
        </button>
      </div>

      <!-- Loop Count (only when no_repeat) -->
      {#if repeatMode === 'no_repeat'}
      <div class="flex items-center gap-2 mb-4">
        <span class="text-sm text-ink-faint">Loop Count</span>
        <input
          type="number"
          bind:value={loopCount}
          min="1"
          max="100"
          class="rounded border px-3 py-2 text-sm width-20 outline-none background-card"
        />
      </div>
      {/if}

      <!-- Save & Clear buttons -->
      <div class="flex gap-2">
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-glorious bg-glorious/10 hover:bg-glorious transition"
          onclick={saveMacro}
          title="Save macro"
        >
          Save Macro
        </button>
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:text-red-400 transition"
          onclick={clearEvents}
          title="Clear all events"
        >
          Clear All
        </button>
      </div>
    </div>

    <!-- Timeline / Action Blocks -->
    <div class="rounded border p-4 bg-panel border-edge">
      <h3 class="text-semibold text-sm mb-3">Macro Timeline</h3>

      {#if timeline.length === 0}
      <p class="text-ink-faint text-sm text-center py-8">No actions yet. Click the add buttons below.</p>
      {:else}
      <div class="space-y-2" role="list">
        {#each timeline as event, index}
        <div
          class="rounded border p-3 flex items-center justify-between bg-card transition"
          style={event.type === 'down' ? 'border-color: #22c55e;' : event.type === 'up' ? 'border-color: #f87171;' : event.type === 'delay' ? 'border-color: #f59e0b;' : 'border-color: a855f7;'}
        >
          <!-- Key Down card -->
          {#if event.type === 'down'}
          <div class="flex items-center gap-2">
            <span class="rounded bg-cyan-500/20 px-2 py-1 text-xs font-medium text-cyan-400">
              ⇩
            </span>
            <span class="text-sm font-medium">{event.key || ''}</span>
          </div>
          <!-- Key Up card -->
          {/if}
          {#if event.type === 'up'}
          <div class="flex items-center gap-2">
            <span class="rounded bg-rose-500/20 px-2 py-1 text-xs font-medium text-rose-400">
              ⇫
            </span>
            <span class="text-sm font-medium">{event.key || ''}</span>
          </div>
          <!-- Delay card -->
          {/if}
          {#if event.type === 'delay'}
          <div class="flex items-center gap-2">
            <span class="rounded bg-amber-500/20 px-2 py-1 text-xs font-medium text-amber-400">
              ⏱
            </span>
            <input
              type="number"
              value={event.delay ?? 0}
              min="0"
              max="10000"
              class="rounded border px-2 py-1 text-sm width-20 outline-none bg-canvas text-amber-300"
              oninput={(e) => (event.delay = parseInt((e.target as HTMLInputElement)?.value || '0') || 0)}
            /> ms
          </div>
          <!-- Text card -->
          {/if}
          {#if event.type === 'text'}
          <div class="flex items-center gap-2">
            <span class="rounded bg-purple-500/20 px-2 py-1 text-xs font-medium text-purple-400">
              Aa
            </span>
            <input
              type="text"
              value={event.text ?? ''}
              class="flex-1 rounded border px-2 py-1 text-sm outline-none bg-canvas text-purple-300"
              oninput={(e) => (event.text = (e.target as HTMLInputElement)?.value || '')}
              placeholder="Enter text..."
            />
          </div>
          {/if}

          <div class="flex gap-1">
            <button
              class="rounded border px-1.5 py-0.5 text-[9px] text-ink-dim hover:text-glorious transition"
              onclick={() => moveEventUp(index)}
              title="Move up"
            >
              ↑
            </button>
            <button
              class="rounded border px-1.5 py-0.5 text-[9px] text-ink-dim hover:text-glorious transition"
              onclick={() => moveEventDown(index)}
              title="Move down"
            >
              ↓
            </button>
            <button
              class="rounded border px-1.5 py-0.5 text-[9px] text-ink-dim hover:text-red-400 transition"
              onclick={() => deleteEvent(index)}
              title="Delete"
            >
              <Trash2 size={10} class="text-red-400" /> D
            </button>
          </div>
        </div>
        {/each}
      </div>
      {/if}

      <!-- Quick Action Adders toolbar -->
      <div class="flex gap-2 mt-4 pt-4 border-t border-edge bg-card">
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 transition"
          onclick={() => timeline.push({ id: crypto.randomUUID(), type: 'down', key: undefined, keyCode: undefined, delay: 0 })}
          title="Key Down"
        >
          + Key Down
        </button>
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 transition"
          onclick={() => timeline.push({ id: crypto.randomUUID(), type: 'up', key: undefined, keyCode: undefined, delay: 0 })}
          title="Key Up"
        >
          + Key Up
        </button>
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 transition"
          onclick={() => timeline.push({ id: crypto.randomUUID(), type: 'delay', delay: 50 })}
          title="+ Delay (50ms)"
        >
          + Delay (50ms)
        </button>
        <button
          class="flex-1 rounded border px-3 py-2 text-sm font-medium text-ink-dim hover:bg-glorious/10 transition"
          onclick={() => timeline.push({ id: crypto.randomUUID(), type: 'text', text: '' })}
          title="+ Text Block"
        >
          + Text Block
        </button>
      </div>
    </div>

    <!-- Test Area / Scratchpad -->
    <div class="rounded border p-4 bg-panel border-edge">
      <h3 class="text-semibold text-sm mb-2">Test / Scratchpad</h3>
      <textarea
        rows="3"
        placeholder="Type here to test or paste keystroke sequences..."
        class="w-full rounded border px-3 py-2 text-sm outline-none bg-card text-ink resize-none"
        oninput={(e) => {
          // Simple test: just show what was typed
          showToast(`Typed: ${(e.target as HTMLTextAreaElement)?.value?.substring(0, 30) ?? ''}...`);
        }}
      ></textarea>
    </div>
  </main>
</div>

{#if toastMessage}
<div
  class="fixed top-4 right-4 z-50 flex items-center gap-2 rounded-full border border-glorious/50 bg-zinc-900/95 px-4 py-2 text-sm font-bold text-glorious shadow-[0_0_24px_rgba(255,184,0,0.4)] backdrop-blur"
>
  <span class="h-2 w-2 rounded-full bg-glorious shadow-[0_0_8px_#ffb800]"></span>
  <span>{toastMessage}</span>
</div>
{/if}
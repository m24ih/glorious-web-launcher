<script lang="ts">
  import type { PinId, ActionCategory, PinAction } from '../lib/types';
  import { DOCK_CATEGORIES, DOCK_ACTIONS, PIN_LABELS } from '../lib/actions';

  interface Props {
    selectedPin: PinId | null;
    mappings: PinAction[];
    onAssign: (category: ActionCategory, action: string) => void;
  }

  let { selectedPin, mappings, onAssign }: Props = $props();

  let activeCategory = $state<ActionCategory>('basickeys');

  const currentMapping = $derived(
    selectedPin ? mappings.find((m) => m.pin === selectedPin) : null
  );

  const currentLabel = $derived(
    selectedPin ? PIN_LABELS[selectedPin] : null
  );
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

    <!-- Action pills grid -->
    <div class="rounded-2xl border border-edge bg-card p-3">
      <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-1.5 max-h-[135px] overflow-y-auto pr-1">
        {#each DOCK_ACTIONS[activeCategory] ?? [] as action (action.id)}
          <button
            type="button"
            class="flex items-center justify-center rounded-xl border border-edge bg-canvas/60 px-2.5 py-2 text-center text-xs font-medium text-ink transition hover:border-glorious hover:bg-glorious/10 hover:text-glorious disabled:cursor-not-allowed disabled:opacity-40"
            disabled={!selectedPin}
            onclick={() => onAssign(activeCategory, action.label)}
            title={selectedPin ? `Assign ${action.label} to ${currentLabel}` : 'Select a pin first'}
          >
            <span class="truncate">{action.label}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

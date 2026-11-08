<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchTraySettings, saveTraySettings } from '../lib/api';
  import type { TraySettings } from '../lib/types';

  let settings = $state<TraySettings | null>(null);
  let error = $state('');
  let saving = $state(false);
  let saved = $state(false);
  const options: { key: keyof TraySettings; label: string; description: string }[] = [
    { key: 'show_dpi', label: 'DPI stages', description: 'Show configured DPI stages in the tray menu.' },
    { key: 'show_profiles', label: 'Profiles', description: 'Switch between device profiles.' },
    { key: 'show_lighting', label: 'Lighting', description: 'Change lighting effects and brightness.' },
    { key: 'show_polling', label: 'Polling rate', description: 'Switch the device polling rate.' },
    { key: 'notifications', label: 'Desktop notifications', description: 'Show connection, charging, and low-battery alerts.' },
  ];

  onMount(async () => {
    try { settings = await fetchTraySettings(); }
    catch (e) { error = e instanceof Error ? e.message : 'Could not load tray settings.'; }
  });

  async function save() {
    if (!settings) return;
    saving = true;
    error = '';
    saved = false;
    try { settings = await saveTraySettings(settings); saved = true; }
    catch (e) { error = e instanceof Error ? e.message : 'Could not save tray settings.'; }
    finally { saving = false; }
  }
</script>

<section class="min-h-0 flex-1 overflow-y-auto px-8 py-7">
  <div class="max-w-2xl">
    <h2 class="text-lg font-bold text-ink">System tray</h2>
    <p class="mt-1 text-sm text-ink-dim">Choose which shortcuts and notifications appear on your desktop.</p>
    {#if error}<p role="alert" class="mt-4 text-sm text-[#ff5c5c]">{error}</p>{/if}
    {#if settings}
      <div class="mt-5 divide-y divide-edge rounded-md border border-edge bg-card">
        {#each options as option (option.key)}
          <label class="flex cursor-pointer items-center justify-between gap-4 px-4 py-4">
            <span><span class="block text-sm font-semibold text-ink">{option.label}</span><span class="mt-1 block text-xs text-ink-dim">{option.description}</span></span>
            <input class="h-4 w-4 accent-[#ffb800]" type="checkbox" bind:checked={settings[option.key]} onchange={() => (saved = false)} />
          </label>
        {/each}
      </div>
      <div class="mt-5 flex items-center gap-3">
        <button type="button" class="rounded-full bg-glorious px-5 py-2 text-xs font-bold text-canvas disabled:opacity-50" disabled={saving} onclick={save}>{saving ? 'Saving…' : 'Save settings'}</button>
        {#if saved}<span role="status" class="text-xs text-ink-dim">Saved</span>{/if}
      </div>
      <p class="mt-4 text-xs text-ink-faint">Tray and notifications are available on Linux. Changes are saved with your launcher configuration.</p>
    {:else if !error}
      <p class="mt-5 text-sm text-ink-dim">Loading…</p>
    {/if}
  </div>
</section>

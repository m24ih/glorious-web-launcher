<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchTraySettings, saveTraySettings } from '../lib/api';
  import type { TraySettings } from '../lib/types';
  import { i18n } from '../lib/i18n.svelte';

  let settings = $state<TraySettings | null>(null);
  let error = $state('');
  let saving = $state(false);
  let saved = $state(false);

  const options: { key: keyof TraySettings; labelKey: string; descKey: string }[] = [
    { key: 'show_dpi', labelKey: 'tray.opt_dpi', descKey: 'tray.opt_dpi_desc' },
    { key: 'show_profiles', labelKey: 'tray.opt_profiles', descKey: 'tray.opt_profiles_desc' },
    { key: 'show_lighting', labelKey: 'tray.opt_lighting', descKey: 'tray.opt_lighting_desc' },
    { key: 'show_polling', labelKey: 'tray.opt_polling', descKey: 'tray.opt_polling_desc' },
    { key: 'notifications', labelKey: 'tray.opt_notifications', descKey: 'tray.opt_notifications_desc' },
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
    <h2 class="text-lg font-bold text-ink">{i18n.t('tray.title')}</h2>
    <p class="mt-1 text-sm text-ink-dim">{i18n.t('tray.subtitle')}</p>
    {#if error}<p role="alert" class="mt-4 text-sm text-[#ff5c5c]">{error}</p>{/if}
    {#if settings}
      <div class="mt-5 divide-y divide-edge rounded-md border border-edge bg-card">
        {#each options as option (option.key)}
          <label class="flex cursor-pointer items-center justify-between gap-4 px-4 py-4">
            <span>
              <span class="block text-sm font-semibold text-ink">{i18n.t(option.labelKey)}</span>
              <span class="mt-1 block text-xs text-ink-dim">{i18n.t(option.descKey)}</span>
            </span>
            <input class="h-4 w-4 accent-[#ffb800]" type="checkbox" bind:checked={settings[option.key]} onchange={() => (saved = false)} />
          </label>
        {/each}
      </div>
      <div class="mt-5 flex items-center gap-3">
        <button
          type="button"
          class="rounded-full bg-glorious px-5 py-2 text-xs font-bold text-canvas disabled:opacity-50"
          disabled={saving}
          onclick={save}
        >
          {saving ? i18n.t('action.saving') : i18n.t('action.save')}
        </button>
        {#if saved}<span role="status" class="text-xs text-ink-dim">{i18n.t('action.saved')}</span>{/if}
      </div>
      <p class="mt-4 text-xs text-ink-faint">{i18n.t('tray.linux_notice')}</p>
    {:else if !error}
      <p class="mt-5 text-sm text-ink-dim">Loading…</p>
    {/if}
  </div>
</section>

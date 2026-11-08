<script lang="ts">
  import { updater } from '../lib/updater.svelte';
  import { i18n } from '../lib/i18n.svelte';
  import { LAUNCHER_VERSION } from '../lib/actions';
  import { Rocket, Sparkles, RefreshCw, AlertCircle, CheckCircle2, X } from 'lucide-svelte';

  const isBusy = $derived(updater.status === 'downloading' || updater.status === 'installing');
</script>

{#if updater.modalOpen}
  <button
    type="button"
    aria-label="Close update dialog"
    class="fixed inset-0 z-40 cursor-default bg-black/70 backdrop-blur-sm"
    onclick={() => { if (!isBusy) updater.closeModal(); }}
    disabled={isBusy}
  ></button>

  <div class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center p-6">
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="update-modal-title"
      class="pointer-events-auto flex max-h-[85vh] w-full max-w-lg flex-col rounded-2xl border border-edge bg-panel p-6 shadow-[0_24px_80px_rgba(0,0,0,0.7)]"
    >
      <!-- Header -->
      <div class="flex items-start justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border border-glorious/30 bg-glorious/10 text-glorious shadow-[0_0_12px_rgba(255,184,0,0.25)]">
            <Rocket size={20} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 id="update-modal-title" class="text-base font-bold text-ink">
                {i18n.t('updater.update_available')}
              </h3>
              <span class="rounded bg-glorious/20 px-2 py-0.5 text-[10px] font-bold text-glorious">
                v{updater.latestVersion || 'latest'}
              </span>
            </div>
            <p class="mt-0.5 text-xs text-ink-dim">
              {i18n.t('updater.current_version', { version: LAUNCHER_VERSION })}
              {#if updater.publishedAt}
                • <span class="text-ink-faint">{updater.publishedAt}</span>
              {/if}
            </p>
          </div>
        </div>

        {#if !isBusy}
          <button
            type="button"
            class="rounded-lg p-1 text-ink-faint transition hover:bg-card hover:text-ink cursor-pointer"
            onclick={() => updater.closeModal()}
            aria-label="Close"
          >
            <X size={18} />
          </button>
        {/if}
      </div>

      <!-- Content / Release Notes -->
      <div class="mt-4 flex-1 overflow-y-auto rounded-xl border border-edge bg-canvas p-3.5 text-xs text-ink-dim max-h-64">
        {#if updater.status === 'downloading'}
          <div class="flex flex-col items-center justify-center py-8 text-center">
            <RefreshCw size={28} class="animate-spin text-glorious" />
            <p class="mt-3 text-sm font-semibold text-ink">{i18n.t('updater.downloading')}</p>
          </div>
        {:else if updater.status === 'installing'}
          <div class="flex flex-col items-center justify-center py-8 text-center">
            <CheckCircle2 size={28} class="text-[#2dff6a]" />
            <p class="mt-3 text-sm font-semibold text-ink">{i18n.t('updater.restarting')}</p>
          </div>
        {:else}
          {#if updater.status === 'error' && updater.errorMessage}
            <div class="mb-3 flex items-center gap-2 rounded-lg border border-[#ff5c5c]/30 bg-[#ff5c5c]/10 p-2.5 text-xs text-[#ff8a8a]">
              <AlertCircle size={16} class="shrink-0" />
              <span>{updater.errorMessage}</span>
            </div>
          {/if}

          <div class="mb-2 font-semibold text-ink uppercase tracking-wider text-[10px]">
            {i18n.t('updater.release_notes')}
          </div>
          {#if updater.releaseNotes}
            <div class="whitespace-pre-wrap leading-relaxed text-ink-dim">
              {updater.releaseNotes}
            </div>
          {:else}
            <p class="italic text-ink-faint">Glorious Web Launcher v{updater.latestVersion}</p>
          {/if}
        {/if}
      </div>

      <!-- Actions -->
      <div class="mt-5 flex items-center justify-end gap-2.5">
        {#if !isBusy}
          <button
            type="button"
            class="rounded-full border border-edge bg-card px-4 py-2 text-xs font-semibold text-ink-dim transition hover:border-edge-bright hover:text-ink cursor-pointer"
            onclick={() => updater.closeModal()}
          >
            {i18n.t('updater.later')}
          </button>
          <button
            type="button"
            class="flex items-center gap-1.5 rounded-full bg-glorious px-5 py-2 text-xs font-bold text-canvas shadow-[0_0_18px_rgba(255,184,0,0.35)] transition hover:bg-glorious-soft cursor-pointer"
            onclick={() => updater.installUpdate()}
          >
            <Sparkles size={14} />
            <span>{i18n.t('updater.update_now')}</span>
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

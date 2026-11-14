<script lang="ts">
  import { onMount } from 'svelte';
  import { Battery, BatteryFull, BatteryLow, BatteryMedium, Bluetooth, Compass, Cpu, Move, MousePointerClick, Palette, Settings, Sliders, Usb, Workflow, Zap, Globe, RefreshCw, Rocket, Sparkles } from 'lucide-svelte';
  import type { DeviceView, DeviceCapabilities, TabId } from '../lib/types';
  import { LAUNCHER_VERSION } from '../lib/actions';
  import { i18n } from '../lib/i18n.svelte';
  import { updater } from '../lib/updater.svelte';
  import UpdateModal from './UpdateModal.svelte';

  interface Props {
    device: DeviceView | null;
    wsStatus: 'connecting' | 'live' | 'offline';
    activeTab: TabId;
    activeProfile: number;
    caps?: DeviceCapabilities;
    onTabChange: (tab: TabId) => void;
    onProfileChange: (profile: number) => void;
    serviceVersion?: string;
  }

  let { device, wsStatus, activeTab, activeProfile, caps, onTabChange, onProfileChange, serviceVersion = LAUNCHER_VERSION }: Props = $props();

  const TABS: Array<{ id: TabId; labelKey: string; icon: typeof Move }> = [
    { id: 'customize', labelKey: 'nav.customize', icon: MousePointerClick },
    { id: 'pointer', labelKey: 'nav.pointer', icon: Move },
    { id: 'lighting', labelKey: 'nav.lighting', icon: Palette },
    { id: 'advanced', labelKey: 'nav.advanced', icon: Sliders },
    { id: 'angle', labelKey: 'nav.angle', icon: Compass },
    { id: 'macro', labelKey: 'nav.macro', icon: Workflow },
    { id: 'firmware', labelKey: 'nav.firmware', icon: Cpu },
    { id: 'tray', labelKey: 'nav.tray', icon: Settings },
  ];

  const visibleTabs = $derived(TABS.filter((t) => !(t.id === 'angle' && !caps?.hasSensorAngle)));

  const statusColor = $derived(
    device?.connected && device.status === 'connected' ? '#2dff6a' : '#6b6b75',
  );
  const connectionMode = $derived(
    device?.connection?.type ??
      (device && device.connected ? 'Wireless' : 'Disconnected'),
  );
  const connectionLabel = $derived(
    connectionMode === 'Wireless'
      ? i18n.t('status.wireless')
      : connectionMode === 'Dongle'
      ? i18n.t('status.dongle')
      : connectionMode === 'Wired'
      ? i18n.t('status.wired')
      : i18n.t('status.disconnected'),
  );
  const batteryPct = $derived(
    device?.battery ? Math.max(device.battery.percentage, 1) : null,
  );

  let checkNotice = $state<{ text: string; isError?: boolean } | null>(null);

  async function handleManualCheck() {
    checkNotice = { text: i18n.t('updater.checking') };
    const hasUpdate = await updater.checkForUpdates(false);
    if (hasUpdate) {
      checkNotice = null;
      updater.openModal();
    } else if (updater.status === 'error') {
      checkNotice = { text: updater.errorMessage || i18n.t('updater.failed'), isError: true };
      setTimeout(() => { checkNotice = null; }, 3500);
    } else {
      checkNotice = { text: i18n.t('updater.up_to_date') };
      setTimeout(() => { checkNotice = null; }, 3500);
    }
  }

  onMount(() => {
    updater.checkForUpdates(true);
  });
</script>

<aside class="flex h-full w-[210px] shrink-0 flex-col overflow-y-auto border-r border-edge bg-panel px-3 py-4">
  <!-- Brand -->
  <div class="flex items-center gap-2 px-1">
    <img src="/brand/logo.png" alt="Glorious" class="h-8 w-10 shrink-0 object-contain" />
    <div class="min-w-0">
      <div class="truncate text-xs font-bold tracking-[0.16em] text-ink">GLORIOUS</div>
      <div class="truncate text-[8px] font-medium tracking-[0.18em] text-ink-faint">WEB LAUNCHER</div>
    </div>
    <div
      class="ml-auto flex shrink-0 items-center gap-1 rounded border border-edge bg-card px-1.5 py-1 text-[9px]"
      title="WebSocket status"
      role="status"
      aria-live="polite"
    >
      <span
        class="h-1.5 w-1.5 rounded-full animate-pulse-soft"
        style="background: {wsStatus === 'live' ? '#2dff6a' : wsStatus === 'connecting' ? '#ffb800' : '#ff5c5c'}"
      ></span>
      <span class="text-ink-dim">{i18n.t(`status.${wsStatus}`)}</span>
    </div>
  </div>

  <!-- Device card -->
  <div class="mt-4 rounded-md border border-edge bg-card p-3">
    {#if device}
      <div class="flex items-center gap-2">
        <span class="h-2 w-2 shrink-0 rounded-full" style="background: {statusColor}; box-shadow: 0 0 8px {statusColor}"></span>
        <span class="min-w-0 truncate text-sm font-semibold text-ink" translate="no">{device.name}</span>
      </div>
      <div class="mt-1 pl-4 text-[11px] text-ink-faint">
        {#if device.error}<span class="text-[#ff5c5c]">{device.error}</span>{:else}{connectionLabel}{/if}
      </div>

      <div class="mt-3 grid gap-2">
        {#if batteryPct !== null}
          <div class="flex items-center gap-2 rounded border border-edge bg-canvas px-2 py-1.5">
            <span class="inline-flex items-center text-glorious">
              {#if device.battery?.charging}
                <Battery size={16} />
              {:else if batteryPct >= 50}
                <BatteryFull size={16} />
              {:else if batteryPct >= 20}
                <BatteryMedium size={16} />
              {:else}
                <BatteryLow size={16} />
              {/if}
            </span>
            <span class="text-xs font-semibold text-ink">{batteryPct}%</span>
            <div class="ml-auto h-1.5 w-14 overflow-hidden rounded-full bg-edge">
              <div
                class="h-full rounded-full"
                style="width: {batteryPct}%; background: {batteryPct < 20 ? '#ff5c5c' : '#ffb800'}"
              ></div>
            </div>
          </div>
        {:else}
          <div class="flex flex-1 items-center gap-2 rounded-full border border-edge bg-canvas px-3 py-1.5 text-xs text-ink-faint">
            {i18n.t('status.no_battery')}
          </div>
        {/if}
        <div
          class="flex items-center gap-1.5 rounded border border-edge bg-canvas px-2 py-1.5 text-[11px] text-ink-dim"
          title="Connection mode"
        >
          {#if connectionMode === 'Dongle' || connectionMode === 'Wireless'}
            <Bluetooth size={14} class="text-glorious" />
          {:else}
            <Usb size={14} class="text-glorious" />
          {/if}
          {connectionLabel}
        </div>
      </div>
    {:else}
      <div class="flex items-center gap-2">
        <Zap size={16} class="text-ink-faint" />
        <span class="text-sm font-semibold text-ink-dim">{i18n.t('status.no_device')}</span>
      </div>
      <div class="mt-1 pl-6 text-[11px] text-ink-faint">
        {i18n.t('status.start_service')}
      </div>
    {/if}
  </div>

  <!-- Profile switch -->
  <div class="mt-4">
    <div class="mb-2 px-1 text-[10px] font-semibold tracking-[0.18em] text-ink-faint">{i18n.t('nav.profile')}</div>
    <div class="grid grid-cols-2 gap-1.5">
      {#each Array.from({ length: caps?.maxProfiles ?? 3 }, (_, i) => i + 1) as n (n)}
        <button
          type="button"
          class="rounded border px-1 py-1.5 text-[10px] font-semibold {activeProfile === n
            ? 'border-[#5468e7] bg-[#4657c9] text-white'
            : 'border-edge bg-card text-ink-dim hover:border-edge-bright hover:text-ink'}"
          onclick={() => onProfileChange(n)}
          aria-pressed={activeProfile === n}
        >
          {i18n.t('nav.profile_n', { n })}
        </button>
      {/each}
    </div>
  </div>

  <!-- Navigation -->
  <nav class="mt-4 flex flex-col gap-1 border-t border-edge pt-3" aria-label="Device settings">
    {#each visibleTabs as tab (tab.id)}
      {@const Icon = tab.icon}
      <button
        type="button"
        class="flex items-center gap-2 rounded px-2.5 py-2 text-xs font-medium {activeTab === tab.id
          ? 'bg-[#4657c9] text-white'
          : 'text-ink-dim hover:bg-card hover:text-ink'}"
        onclick={() => onTabChange(tab.id)}
        aria-current={activeTab === tab.id ? 'page' : undefined}
      >
        <Icon size={15} class={activeTab === tab.id ? 'text-white' : 'text-ink-faint'} />
        {i18n.t(tab.labelKey)}
      </button>
    {/each}
  </nav>

  <!-- Language Switcher & Versions Footer -->
  <div class="mt-auto flex flex-col gap-2 border-t border-edge px-1 pt-3">
    <div class="flex items-center gap-2">
      <a
        href="https://github.com/m24ih/glorious-web-launcher"
        target="_blank"
        rel="noopener noreferrer"
        title="GitHub"
        aria-label="GitHub Repository"
        class="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-edge bg-card text-ink-dim transition hover:border-glorious/50 hover:text-ink"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
        </svg>
      </a>
      <a
        href="https://gitlab.com/m24ih/glorious-web-launcher"
        target="_blank"
        rel="noopener noreferrer"
        title="GitLab"
        aria-label="GitLab Repository"
        class="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-edge bg-card text-ink-dim transition hover:border-glorious/50 hover:text-ink"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <path d="M23.955 13.587l-1.342-4.135-2.664-8.189a.455.455 0 0 0-.867 0L16.418 9.45H7.582L4.918 1.263a.455.455 0 0 0-.867 0L1.387 9.452.045 13.587a.924.924 0 0 0 .331 1.03L12 23.054l11.624-8.437a.92.92 0 0 0 .331-1.03"/>
        </svg>
      </a>
    </div>

    <div class="flex items-center justify-between rounded border border-edge bg-card px-2 py-1 text-[11px]">
      <div class="flex items-center gap-1.5 text-ink-faint">
        <Globe size={13} class="text-glorious" />
        <span class="font-medium text-[10px]">{i18n.t('nav.language')}</span>
      </div>
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="rounded px-1.5 py-0.5 text-[10px] font-bold transition {i18n.locale === 'tr' ? 'bg-glorious text-canvas' : 'text-ink-dim hover:text-ink'}"
          onclick={() => i18n.setLocale('tr')}
        >
          TR
        </button>
        <span class="text-ink-faint text-[9px]">|</span>
        <button
          type="button"
          class="rounded px-1.5 py-0.5 text-[10px] font-bold transition {i18n.locale === 'en' ? 'bg-glorious text-canvas' : 'text-ink-dim hover:text-ink'}"
          onclick={() => i18n.setLocale('en')}
        >
          EN
        </button>
      </div>
    </div>

    {#if updater.updateAvailable}
      <button
        type="button"
        class="group flex items-center justify-between gap-1 rounded-lg border border-glorious/50 bg-glorious/15 px-2 py-1.5 text-[11px] font-bold text-glorious shadow-[0_0_12px_rgba(255,184,0,0.2)] transition hover:bg-glorious hover:text-canvas cursor-pointer animate-pulse-soft"
        onclick={() => updater.openModal()}
        title="{i18n.t('updater.update_available')}"
      >
        <span class="flex items-center gap-1.5 truncate">
          <span>🚀</span>
          <span class="truncate">{i18n.t('updater.new_version')}</span>
          <span class="font-mono text-[10px]">v{updater.latestVersion}</span>
        </span>
        <Sparkles size={13} class="shrink-0" />
      </button>
    {/if}

    <div class="flex flex-col gap-0.5 text-[9px] text-ink-faint">
      <div class="flex items-center justify-between gap-1">
        <button
          type="button"
          class="flex items-center gap-1 text-left transition hover:text-ink cursor-pointer"
          onclick={handleManualCheck}
          title="{i18n.t('updater.check')}"
        >
          <span>Launcher v{LAUNCHER_VERSION}</span>
          <RefreshCw
            size={9}
            class={updater.status === 'checking' ? 'animate-spin text-glorious' : 'text-ink-faint hover:text-ink'}
          />
        </button>
        <span>Service v{serviceVersion || '—'}</span>
      </div>
      {#if checkNotice}
        <div class="truncate text-[8.5px] font-medium {checkNotice.isError ? 'text-[#ff5c5c]' : 'text-glorious'}">
          {checkNotice.text}
        </div>
      {/if}
    </div>
  </div>

  <UpdateModal />
</aside>

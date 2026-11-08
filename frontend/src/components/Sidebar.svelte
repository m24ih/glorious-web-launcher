<script lang="ts">
  import { Battery, BatteryFull, BatteryLow, BatteryMedium, Bluetooth, Compass, Cpu, Move, MousePointerClick, Palette, Settings, Sliders, Usb, Workflow, Zap, Globe } from 'lucide-svelte';
  import type { DeviceView, DeviceCapabilities, TabId } from '../lib/types';
  import { LAUNCHER_VERSION } from '../lib/actions';
  import { i18n } from '../lib/i18n.svelte';

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

    <div class="flex flex-col gap-0.5 text-[9px] text-ink-faint">
      <span>Launcher v{LAUNCHER_VERSION}</span>
      <span>Service v{serviceVersion || '—'}</span>
    </div>
  </div>
</aside>

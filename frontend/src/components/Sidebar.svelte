<script lang="ts">
  import { Battery, BatteryFull, BatteryLow, BatteryMedium, Bluetooth, Compass, Cpu, Move, MousePointerClick, Palette, Sliders, Usb, Workflow, Zap } from 'lucide-svelte';
  import type { DeviceView, DeviceCapabilities, TabId } from '../lib/types';
  import { LAUNCHER_VERSION } from '../lib/actions';

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

  const TABS: Array<{ id: TabId; label: string; icon: typeof Move }> = [
    { id: 'customize', label: 'Customize', icon: MousePointerClick },
    { id: 'pointer', label: 'Pointer', icon: Move },
    { id: 'lighting', label: 'Lighting', icon: Palette },
    { id: 'advanced', label: 'Advanced', icon: Sliders },
    { id: 'angle', label: 'Angle Adjustment', icon: Compass },
    { id: 'macro', label: 'Macro', icon: Workflow },
    { id: 'firmware', label: 'Firmware', icon: Cpu },
  ];

  const visibleTabs = $derived(TABS.filter((t) => !(t.id === 'angle' && !caps?.hasSensorAngle)));

  const statusColor = $derived(
    device?.connected && device.status === 'connected' ? '#2dff6a' : '#6b6b75',
  );
  const connectionMode = $derived(
    device?.connection?.type ??
      (device && device.connected ? 'Wireless' : 'Disconnected'),
  );
  const batteryPct = $derived(
    device?.battery ? Math.max(device.battery.percentage, 1) : null,
  );
</script>

<aside class="flex h-full w-[320px] shrink-0 flex-col gap-4 border-r border-edge bg-panel px-5 py-6">
  <!-- Brand -->
  <div class="flex items-center gap-3 px-1">
    <img src="/brand/logo.png" alt="Glorious" class="h-10 w-14 object-contain" />
    <div>
      <div class="text-sm font-bold tracking-[0.22em] text-ink">GLORIOUS</div>
      <div class="text-[10px] font-medium tracking-[0.3em] text-ink-faint">WEB LAUNCHER</div>
    </div>
    <div
      class="ml-auto flex items-center gap-1.5 rounded-full border border-edge bg-card px-2.5 py-1 text-[10px]"
      title="WebSocket status"
    >
      <span
        class="h-1.5 w-1.5 rounded-full animate-pulse-soft"
        style="background: {wsStatus === 'live' ? '#2dff6a' : wsStatus === 'connecting' ? '#ffb800' : '#ff5c5c'}"
      ></span>
      <span class="text-ink-dim">{wsStatus}</span>
    </div>
  </div>

  <!-- Device card -->
  <div class="rounded-2xl border border-edge bg-card p-4">
    {#if device}
      <div class="flex items-center gap-2">
        <span class="h-2 w-2 shrink-0 rounded-full" style="background: {statusColor}; box-shadow: 0 0 8px {statusColor}"></span>
        <span class="min-w-0 truncate text-sm font-semibold text-ink" translate="no">{device.name}</span>
      </div>
      <div class="mt-1 pl-4 text-[11px] text-ink-faint">
        {#if device.error}<span class="text-[#ff5c5c]">{device.error}</span>{:else}{connectionMode}{/if}
      </div>

      <div class="mt-3 flex items-center gap-2">
        {#if batteryPct !== null}
          <div class="flex flex-1 items-center gap-2 rounded-full border border-edge bg-canvas px-3 py-1.5">
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
            No battery data
          </div>
        {/if}
        <div
          class="flex items-center gap-1.5 rounded-full border border-edge bg-canvas px-3 py-1.5 text-xs text-ink-dim"
          title="Connection mode"
        >
          {#if connectionMode === 'Dongle' || connectionMode === 'Wireless'}
            <Bluetooth size={14} class="text-glorious" />
          {:else}
            <Usb size={14} class="text-glorious" />
          {/if}
          {connectionMode}
        </div>
      </div>
    {:else}
      <div class="flex items-center gap-2">
        <Zap size={16} class="text-ink-faint" />
        <span class="text-sm font-semibold text-ink-dim">No device available</span>
      </div>
      <div class="mt-1 pl-6 text-[11px] text-ink-faint">
        Start glorious-service to detect mice.
      </div>
    {/if}
  </div>

  <!-- Profile switch -->
  <div>
    <div class="mb-2 px-1 text-[10px] font-semibold tracking-[0.18em] text-ink-faint">PROFILE</div>
    <div class="flex gap-2">
      {#each Array.from({ length: caps?.maxProfiles ?? 3 }, (_, i) => i + 1) as n (n)}
        <button
          type="button"
          class="flex-1 rounded-full border py-1.5 text-xs font-semibold {activeProfile === n
            ? 'border-glorious bg-glorious/10 text-glorious'
            : 'border-edge bg-card text-ink-dim hover:border-edge-bright hover:text-ink'}"
          onclick={() => onProfileChange(n)}
        >
          Profile {n}
        </button>
      {/each}
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex flex-col gap-1 border-t border-edge pt-4">
    {#each visibleTabs as tab (tab.id)}
      {@const Icon = tab.icon}
      <button
        type="button"
        class="flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium {activeTab === tab.id
          ? 'bg-card text-glorious shadow-[inset 2px 0 0_#ffb800]'
          : 'text-ink-dim hover:bg-card hover:text-ink'}"
        onclick={() => onTabChange(tab.id)}
      >
        <Icon size={17} class={activeTab === tab.id ? 'text-glorious' : 'text-ink-faint'} />
        {tab.label}
      </button>
    {/each}
  </nav>

  <div class="mt-auto flex items-center justify-between px-1 text-[10px] text-ink-faint">
    <span>Launcher v{LAUNCHER_VERSION}</span>
    <span>Service v{serviceVersion || '—'}</span>
  </div>
</aside>

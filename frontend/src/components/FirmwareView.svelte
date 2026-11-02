<script lang="ts">
  import type { DeviceView } from '../lib/types';
  import { Cpu, Usb, Battery, BatteryCharging, ShieldCheck, Activity } from 'lucide-svelte';

  interface Props {
    device: DeviceView | null;
  }

  let { device }: Props = $props();

  const batteryPct = $derived(
    device?.battery ? Math.max(device.battery.percentage, 1) : null
  );

  const isCharging = $derived(device?.battery?.charging ?? false);

  const protocolDisplayName = $derived(
    device?.protocol === 'bamf2' || device?.protocol === 'pixart'
      ? 'Pixart BAMF 2.0'
      : device?.protocol === 'sinowealth'
        ? 'Sinowealth'
        : (device?.protocol ?? 'Glorious Core Protocol')
  );
</script>

<div class="flex-1 overflow-y-auto p-8">
  <div class="mx-auto max-w-[800px] space-y-6">
    <!-- Header -->
    <div>
      <h2 class="text-xl font-bold text-ink">Hardware & Firmware Telemetry</h2>
      <p class="text-xs text-ink-dim">Real-time device diagnostics, protocol layer details, and firmware revisions.</p>
    </div>

    {#if device}
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <!-- Device Model & Slug -->
        <div class="rounded-2xl border border-edge bg-card p-5">
          <div class="flex items-center gap-2 mb-2 text-ink-faint">
            <Cpu size={16} class="text-glorious" />
            <span class="text-[10px] font-bold tracking-[0.18em] uppercase">Device Model</span>
          </div>
          <div class="text-lg font-bold text-ink">{device.name}</div>
          <div class="mt-1 font-mono text-xs text-ink-dim">Slug: <span class="text-ink">{device.slug}</span></div>
          <div class="mt-1 text-[11px] text-ink-faint">Router ID: {device.routerID || '—'}</div>
        </div>

        <!-- Protocol Card -->
        <div class="rounded-2xl border border-edge bg-card p-5">
          <div class="flex items-center gap-2 mb-2 text-ink-faint">
            <Activity size={16} class="text-glorious" />
            <span class="text-[10px] font-bold tracking-[0.18em] uppercase">Protocol Architecture</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="h-2.5 w-2.5 rounded-full bg-glorious shadow-[0_0_8px_#ffb800]"></span>
            <span class="text-base font-bold text-ink">{protocolDisplayName}</span>
          </div>
          <div class="mt-2 text-xs text-ink-dim">
            {#if device.protocol === 'bamf2'}
              High-efficiency BAMF 2.0 sensor engine (26,000 DPI / 650 IPS).
            {:else if device.protocol === 'sinowealth'}
              Sinowealth 8-bit MCU HID reporting interface.
            {:else}
              Hardware abstraction driver active.
            {/if}
          </div>
        </div>

        <!-- Connection & Interface -->
        <div class="rounded-2xl border border-edge bg-card p-5">
          <div class="flex items-center gap-2 mb-2 text-ink-faint">
            <Usb size={16} class="text-glorious" />
            <span class="text-[10px] font-bold tracking-[0.18em] uppercase">Connection & USB HID</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm font-semibold text-ink">{device.connection?.type ?? (device.connected ? 'Connected' : 'Disconnected')}</span>
            <span class="rounded-full border border-edge bg-canvas px-2.5 py-0.5 text-xs font-mono text-ink-dim">
              VID:PID {device.connection?.vid ?? '—'}:{device.connection?.pid ?? '—'}
            </span>
          </div>
          <div class="mt-3 space-y-1 text-xs text-ink-dim">
            <div>Interface: <span class="font-mono text-ink">{device.connection?.interface ?? '0'}</span></div>
            <div class="break-all font-mono text-[10px] text-ink-faint">Path: {device.connection?.path ?? '—'}</div>
          </div>
        </div>

        <!-- Battery Telemetry -->
        <div class="rounded-2xl border border-edge bg-card p-5">
          <div class="flex items-center gap-2 mb-2 text-ink-faint">
            {#if isCharging}
              <BatteryCharging size={16} class="text-glorious" />
            {:else}
              <Battery size={16} class="text-glorious" />
            {/if}
            <span class="text-[10px] font-bold tracking-[0.18em] uppercase">Battery Telemetry</span>
          </div>
          {#if batteryPct !== null}
            <div class="flex items-baseline gap-2">
              <span class="text-3xl font-extrabold text-ink">{batteryPct}%</span>
              <span class="text-xs font-bold {isCharging ? 'text-glorious' : 'text-ink-dim'}">
                {isCharging ? 'Charging' : (device.battery?.status ?? 'Discharging')}
              </span>
            </div>
            <div class="mt-3 h-2 w-full overflow-hidden rounded-full bg-edge">
              <div
                class="h-full rounded-full bg-glorious shadow-[0_0_12px_rgba(255,184,0,0.4)]"
                style="width: {batteryPct}%"
              ></div>
            </div>
          {:else}
            <div class="text-sm text-ink-faint">No wireless battery telemetry available (Wired mode)</div>
          {/if}
        </div>

        <!-- Firmware Info -->
        <div class="col-span-1 sm:col-span-2 rounded-2xl border border-edge bg-card p-5">
          <div class="flex items-center gap-2 mb-3 text-ink-faint">
            <ShieldCheck size={16} class="text-glorious" />
            <span class="text-[10px] font-bold tracking-[0.18em] uppercase">Firmware Revisions</span>
          </div>
          <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <div class="rounded-xl border border-edge bg-canvas/60 p-3 text-center">
              <div class="text-[10px] font-semibold text-ink-faint uppercase">Display FW</div>
              <div class="mt-1 font-mono text-sm font-bold text-ink">{device.firmware?.display ?? '—'}</div>
            </div>
            <div class="rounded-xl border border-edge bg-canvas/60 p-3 text-center">
              <div class="text-[10px] font-semibold text-ink-faint uppercase">Mouse FW</div>
              <div class="mt-1 font-mono text-sm font-bold text-ink">{device.firmware?.mouse ?? '—'}</div>
            </div>
            <div class="rounded-xl border border-edge bg-canvas/60 p-3 text-center">
              <div class="text-[10px] font-semibold text-ink-faint uppercase">Dongle FW</div>
              <div class="mt-1 font-mono text-sm font-bold text-ink">{device.firmware?.dongle ?? '—'}</div>
            </div>
          </div>
        </div>
      </div>
    {:else}
      <div class="rounded-2xl border border-edge bg-card p-12 text-center">
        <Cpu size={32} class="mx-auto mb-3 text-ink-faint" />
        <h3 class="text-base font-bold text-ink">No Device Connected</h3>
        <p class="mt-1 text-xs text-ink-dim">Connect a Glorious mouse or start glorious-service to view telemetry.</p>
      </div>
    {/if}
  </div>
</div>

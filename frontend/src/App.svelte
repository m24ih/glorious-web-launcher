<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Stage from './components/Stage.svelte';
  import Dock from './components/Dock.svelte';
  import PointerView from './components/PointerView.svelte';
  import LightingView from './components/LightingView.svelte';
  import FirmwareView from './components/FirmwareView.svelte';
  import type {
    DeviceView,
    PinAction,
    PinId,
    ActionCategory,
    SettingsPatch,
    TabId,
    SettingsDraft,
    RgbEffectName,
  } from './lib/types';
  import {
    fetchDevices,
    applySettings,
    connectWs,
  } from './lib/api';
  import {
    DEFAULT_PIN_ACTIONS,
    draftFromDevice,
    hexToRgb,
    LIGHTING_EFFECTS,
    LAUNCHER_VERSION,
  } from './lib/actions';

  // State management
  let device = $state<DeviceView | null>(null);
  let wsStatus = $state<'connecting' | 'live' | 'offline'>('connecting');
  let activeTab = $state<TabId>('customize');
  let activeProfile = $state<number>(1);
  let selectedPin = $state<PinId | null>('left');
  let mappings = $state<PinAction[]>(DEFAULT_PIN_ACTIONS);
  let draft = $state<SettingsDraft>(draftFromDevice(null));
  let applying = $state<boolean>(false);
  let toastMessage = $state<string | null>(null);

  const canApply = $derived(device !== null && !applying);

  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  function showToast(msg: string) {
    toastMessage = msg;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastMessage = null;
    }, 3200);
  }

  function onAssign(category: ActionCategory, action: string) {
    if (!selectedPin) return;
    mappings = mappings.map((m) =>
      m.pin === selectedPin ? { ...m, category, action } : m,
    );
    showToast(`Rebound ${selectedPin} to ${action}`);
  }

  function onDraftChange(updater: (draft: SettingsDraft) => SettingsDraft) {
    draft = updater(draft);
  }

  function onReset() {
    draft = draftFromDevice(device);
    mappings = DEFAULT_PIN_ACTIONS;
    showToast('Reverted settings to defaults');
  }

  async function onApply() {
    if (!device || applying) return;
    applying = true;
    try {
      const effectSpec = LIGHTING_EFFECTS.find((e) => e.id === draft.lighting.effect);
      const rgbEffect: RgbEffectName = (effectSpec?.name as RgbEffectName) ?? 'glorious';
      const rgbColor = hexToRgb(draft.lighting.color);

      const patch: SettingsPatch = {
        profile: activeProfile,
        dpi_stages: draft.dpi_stages.map((s) => s.value),
        active_dpi_stage: draft.active_dpi_stage,
        polling_rate: draft.polling_rate,
        debounce_ms: draft.debounce_ms,
        lod: draft.lod,
        sleep_seconds: draft.sleep_seconds === 'off' ? 0 : draft.sleep_seconds,
        rgb: {
          brightness: draft.lighting.brightness,
          effect: rgbEffect,
          rate: draft.lighting.rate,
          color: rgbColor,
        },
      };

      const res = await applySettings(device.slug, patch);
      if (res?.device) {
        device = res.device;
        draft = draftFromDevice(res.device);
      }
      showToast('Settings applied to mouse');
    } catch (e: any) {
      showToast(e?.message ?? 'Failed to apply settings to device');
    } finally {
      applying = false;
    }
  }

  onMount(() => {
    wsStatus = 'connecting';

    fetchDevices()
      .then((list) => {
        if (list.devices.length > 0) {
          const connected = list.devices.find((d) => d.connected) ?? list.devices[0];
          device = connected;
          draft = draftFromDevice(connected);
          if (connected.settings?.active_profile) {
            activeProfile = connected.settings.active_profile;
          }
        }
      })
      .catch((e) => {
        console.warn('Initial device fetch failed, waiting for WebSocket:', e);
      });

    const unsubWs = connectWs(
      (event) => {
        if (event.type === 'snapshot' || event.type === 'telemetry') {
          if (event.devices.length > 0) {
            const currentSlug = device?.slug;
            const updated = currentSlug
              ? event.devices.find((d) => d.slug === currentSlug)
              : null;
            const chosen = updated ?? event.devices.find((d) => d.connected) ?? event.devices[0];
            device = chosen;
            if (!device || !updated) {
              draft = draftFromDevice(chosen);
            }
          }
        } else if (event.type === 'applied') {
          if (device && event.slug === device.slug) {
            device = event.device;
            draft = draftFromDevice(event.device);
          }
          showToast('Settings synced from hardware');
        }
      },
      (status) => {
        wsStatus = status;
      },
    );

    return () => {
      unsubWs();
      if (toastTimer) clearTimeout(toastTimer);
    };
  });
</script>

<div class="flex h-screen w-screen overflow-hidden bg-canvas text-ink select-none">
  <!-- Left Sidebar matching Keychron BM24 + Glorious palette -->
  <Sidebar
    {device}
    {wsStatus}
    {activeTab}
    {activeProfile}
    onTabChange={(tab) => (activeTab = tab)}
    onProfileChange={(n) => {
      activeProfile = n;
      draft = { ...draft, profile: n };
    }}
    serviceVersion={LAUNCHER_VERSION}
  />

  <!-- Main View Area -->
  <main class="flex min-w-0 flex-1 flex-col bg-canvas overflow-hidden">
    {#if activeTab === 'customize'}
      <div class="flex min-h-0 flex-1 flex-col p-6">
        <Stage
          {device}
          {mappings}
          selected={selectedPin}
          {canApply}
          {applying}
          onSelect={(pin) => (selectedPin = selectedPin === pin ? null : pin)}
          {onReset}
          {onApply}
        />
      </div>
      <Dock
        {selectedPin}
        {mappings}
        {onAssign}
      />
    {:else}
      <!-- Secondary Tab Navigation Header -->
      <div class="flex items-center justify-between border-b border-edge bg-panel px-8 py-4">
        <div class="flex items-center gap-3">
          <h1 class="text-base font-bold text-ink truncate">{device?.name ?? 'Glorious Device'}</h1>
          {#if device?.battery}
            <span class="rounded-full border border-edge bg-card px-2.5 py-0.5 text-xs font-semibold text-glorious">
              {Math.max(device.battery.percentage, 1)}%
            </span>
          {/if}
          <span class="text-xs text-ink-dim capitalize">· {activeTab}</span>
        </div>

        {#if activeTab !== 'firmware'}
          <div class="flex items-center gap-2">
            <button
              type="button"
              class="flex items-center gap-2 rounded-full border border-edge bg-card px-4 py-2 text-xs font-semibold text-ink-dim hover:border-edge-bright hover:text-ink transition"
              onclick={onReset}
            >
              Reset
            </button>
            <button
              type="button"
              class="flex items-center gap-2 rounded-full bg-glorious px-5 py-2 text-xs font-bold text-canvas shadow-[0_0_18px_rgba(255,184,0,0.35)] hover:bg-glorious-soft disabled:cursor-not-allowed disabled:opacity-40 disabled:shadow-none transition"
              disabled={!canApply}
              onclick={onApply}
            >
              {applying ? 'Applying…' : 'Apply to Mouse'}
            </button>
          </div>
        {/if}
      </div>

      <!-- Tab Content Area -->
      {#if activeTab === 'pointer'}
        <PointerView {draft} onChange={onDraftChange} />
      {:else if activeTab === 'lighting'}
        <LightingView {draft} onChange={onDraftChange} />
      {:else if activeTab === 'firmware'}
        <FirmwareView {device} />
      {/if}
    {/if}

    <!-- Toast Notification -->
    {#if toastMessage}
      <div class="fixed bottom-6 right-6 z-50 flex items-center gap-2 rounded-full border border-glorious/50 bg-zinc-900/95 px-5 py-2.5 text-xs font-bold text-glorious shadow-[0_0_24px_rgba(255,184,0,0.4)] backdrop-blur">
        <span class="h-2 w-2 rounded-full bg-glorious shadow-[0_0_8px_#ffb800]"></span>
        <span>{toastMessage}</span>
      </div>
    {/if}
  </main>
</div>

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Stage from './components/Stage.svelte';
  import Dock from './components/Dock.svelte';
  import PointerView from './components/PointerView.svelte';
  import LightingView from './components/LightingView.svelte';
  import FirmwareView from './components/FirmwareView.svelte';
  import MacroView from './components/MacroView.svelte';
  import type {
    DeviceView,
    PinAction,
    PinId,
    ActionCategory,
    SettingsPatch,
    TabId,
    LightingSettings,
    SettingsDraft,
  } from './lib/types';
  import {
    fetchDevices,
    applySettings,
    connectWs,
  } from './lib/api';
  import {
    DEFAULT_PIN_ACTIONS,
    DPI_COLORS,
    draftFromDevice,
    draftFromSettings,
    hexToRgb,
    LIGHTING_EFFECTS,
    LAUNCHER_VERSION,
    pinActionsToBindings,
    bindingsToPinActions,
  } from './lib/actions';

  // State management
  let device = $state<DeviceView | null>(null);
  let wsStatus = $state<'connecting' | 'live' | 'offline'>('connecting');
  let activeTab = $state<TabId>('customize');
  let activeProfile = $state<number>(1);
  let selectedPin = $state<PinId | null>('left');
  let mappings = $state<PinAction[]>(DEFAULT_PIN_ACTIONS);
  let draft = $state<SettingsDraft>(draftFromDevice(null));
  let profileDrafts = $state<Record<number, { draft: SettingsDraft; mappings: PinAction[] }>>({});
  let hasUserModified = $state<boolean>(false);
  let applying = $state<boolean>(false);
  let toastMessage = $state<string | null>(null);

  const canApply = $derived(device !== null && !applying);

  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  let profileDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  function showToast(msg: string) {
    toastMessage = msg;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastMessage = null;
    }, 3200);
  }

  function populateProfileDrafts(dev: DeviceView) {
    if (dev.profiles) {
      for (const [pidStr, settings] of Object.entries(dev.profiles)) {
        const pid = Number(pidStr);
        if (!profileDrafts[pid] || !hasUserModified) {
          profileDrafts[pid] = {
            draft: draftFromSettings(settings),
            mappings: settings.buttons && settings.buttons.length > 0
              ? bindingsToPinActions(settings.buttons)
              : DEFAULT_PIN_ACTIONS,
          };
        }
      }
    }
  }

  function onProfileChange(newProfile: number) {
    if (newProfile === activeProfile) return;

    // Save current profile's draft and mappings
    profileDrafts[activeProfile] = {
      draft: $state.snapshot(draft),
      mappings: [...mappings],
    };

    activeProfile = newProfile;

    if (profileDrafts[newProfile]) {
      draft = profileDrafts[newProfile].draft;
      mappings = profileDrafts[newProfile].mappings;
    } else if (device?.profiles?.[newProfile]) {
      const pSettings = device.profiles[newProfile];
      draft = draftFromSettings(pSettings);
      mappings = pSettings.buttons && pSettings.buttons.length > 0
        ? bindingsToPinActions(pSettings.buttons)
        : DEFAULT_PIN_ACTIONS;
      profileDrafts[newProfile] = {
        draft: $state.snapshot(draft),
        mappings: [...mappings],
      };
    } else {
      draft = { ...draftFromSettings(null), profile: newProfile };
      mappings = DEFAULT_PIN_ACTIONS;
      profileDrafts[newProfile] = {
        draft: $state.snapshot(draft),
        mappings: [...mappings],
      };
    }

    if (profileDebounceTimer) {
      clearTimeout(profileDebounceTimer);
      profileDebounceTimer = null;
    }

    if (device?.slug) {
      const slug = device.slug;
      profileDebounceTimer = setTimeout(async () => {
        try {
          const res = await applySettings(slug, { profile: newProfile });
          if (res?.device) {
            device = res.device;
            populateProfileDrafts(res.device);
          }
          showToast(`Switched mouse to Profile ${newProfile}`);
        } catch (e: any) {
          console.warn('Failed to switch profile on device:', e);
          showToast(e?.message ?? `Failed to switch to Profile ${newProfile}`);
        }
      }, 150);
    }
  }

  function onAssign(category: ActionCategory, action: string) {
    if (!selectedPin) return;
    hasUserModified = true;
    mappings = mappings.map((m) =>
      m.pin === selectedPin ? { ...m, category, action } : m,
    );
    profileDrafts[activeProfile] = {
      draft: $state.snapshot(draft),
      mappings: [...mappings],
    };
    showToast(`Rebound ${selectedPin} to ${action}`);
  }

  function onDraftChange(updater: (draft: SettingsDraft) => SettingsDraft) {
    hasUserModified = true;
    draft = updater(draft);
    profileDrafts[activeProfile] = {
      draft: $state.snapshot(draft),
      mappings: [...mappings],
    };
  }

  async function onLightingLivePreview(lighting: LightingSettings) {
    if (!device) return;
    const rgbColor = hexToRgb(lighting.color);
    if (device.settings) {
      device.settings = {
        ...device.settings,
        lighting: {
          effect: lighting.effect,
          brightness: lighting.brightness,
          rate: lighting.rate,
          color: rgbColor,
        },
      };
    }
    try {
      const patch: SettingsPatch = {
        rgb: {
          effect: lighting.effect,
          brightness: lighting.brightness,
          rate: lighting.rate,
          color: rgbColor,
        },
      };
      await applySettings(device.slug, patch);
    } catch (e) {
      console.warn('Live lighting preview failed:', e);
    }
  }

  function onReset() {
    hasUserModified = false;
    if (device?.profiles?.[activeProfile]) {
      const p = device.profiles[activeProfile];
      draft = draftFromSettings(p);
      mappings = p.buttons && p.buttons.length > 0
        ? bindingsToPinActions(p.buttons)
        : DEFAULT_PIN_ACTIONS;
    } else if (device?.settings && activeProfile === (device.settings.active_profile ?? 1)) {
      draft = draftFromDevice(device);
      mappings = device.settings.buttons && device.settings.buttons.length > 0
        ? bindingsToPinActions(device.settings.buttons)
        : DEFAULT_PIN_ACTIONS;
    } else {
      draft = { ...draftFromSettings(null), profile: activeProfile };
      mappings = DEFAULT_PIN_ACTIONS;
    }
    profileDrafts[activeProfile] = {
      draft: $state.snapshot(draft),
      mappings: [...mappings],
    };
    showToast('Reverted settings to defaults');
  }

  async function onApply() {
    if (!device || applying) return;
    applying = true;
    try {
      const rgbColor = hexToRgb(draft.lighting.color);

      const patch: SettingsPatch = {
        profile: activeProfile,
        dpi_stages: draft.dpi_stages.map((s) => s.value),
        dpi_colors: draft.dpi_stages.map((s, i) => s.color || DPI_COLORS[i % DPI_COLORS.length]),
        active_dpi_stage: draft.active_dpi_stage,
        polling_rate: draft.polling_rate,
        debounce_ms: draft.debounce_ms,
        lod: draft.lod,
        sleep_seconds: draft.sleep_seconds === 'off' ? 0 : draft.sleep_seconds,
        buttons: pinActionsToBindings(mappings),
        rgb: {
          brightness: draft.lighting.brightness,
          effect: draft.lighting.effect,
          rate: draft.lighting.rate,
          color: rgbColor,
        },
      };

      const res = await applySettings(device.slug, patch);
      if (res?.device) {
        device = res.device;
        populateProfileDrafts(res.device);
        if (profileDrafts[activeProfile]) {
          draft = profileDrafts[activeProfile].draft;
          mappings = profileDrafts[activeProfile].mappings;
        } else if (res.device.settings) {
          draft = draftFromSettings(res.device.settings);
          if (res.device.settings.buttons && res.device.settings.buttons.length > 0) {
            mappings = bindingsToPinActions(res.device.settings.buttons);
          }
        }
        hasUserModified = false;
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
          populateProfileDrafts(connected);
          if (connected.settings?.active_profile) {
            activeProfile = connected.settings.active_profile;
          }
          if (!hasUserModified) {
            if (profileDrafts[activeProfile]) {
              draft = profileDrafts[activeProfile].draft;
              mappings = profileDrafts[activeProfile].mappings;
            } else {
              draft = draftFromDevice(connected);
              if (connected.settings?.buttons && connected.settings.buttons.length > 0) {
                mappings = bindingsToPinActions(connected.settings.buttons);
              }
            }
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
            populateProfileDrafts(chosen);
            if (!hasUserModified) {
              if (profileDrafts[activeProfile]) {
                draft = profileDrafts[activeProfile].draft;
                mappings = profileDrafts[activeProfile].mappings;
              } else {
                draft = draftFromDevice(chosen);
                if (chosen.settings?.buttons && chosen.settings.buttons.length > 0) {
                  mappings = bindingsToPinActions(chosen.settings.buttons);
                }
              }
            }
          }
        } else if (event.type === 'applied') {
          if (device && event.slug === device.slug) {
            device = event.device;
            populateProfileDrafts(event.device);
            if (!hasUserModified) {
              if (profileDrafts[activeProfile]) {
                draft = profileDrafts[activeProfile].draft;
                mappings = profileDrafts[activeProfile].mappings;
              } else {
                draft = draftFromDevice(event.device);
                if (event.device.settings?.buttons && event.device.settings.buttons.length > 0) {
                  mappings = bindingsToPinActions(event.device.settings.buttons);
                }
              }
            }
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
      if (profileDebounceTimer) clearTimeout(profileDebounceTimer);
    };
  });

  onDestroy(() => {
    if (profileDebounceTimer) clearTimeout(profileDebounceTimer);
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
    onProfileChange={onProfileChange}
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
        <PointerView {device} {draft} onChange={onDraftChange} />
      {:else if activeTab === 'lighting'}
        <LightingView {draft} onChange={onDraftChange} onLivePreview={onLightingLivePreview} />
      {:else if activeTab === 'firmware'}
        <FirmwareView {device} />
      {:else if activeTab === 'macro'}
        <MacroView {activeTab} onTabChange={(t: string) => (activeTab = t as TabId)} />
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

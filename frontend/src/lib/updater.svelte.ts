import { LAUNCHER_VERSION } from './actions';

export type UpdateStatus = 'idle' | 'checking' | 'downloading' | 'installing' | 'error';

export function parseSemver(v: string): number[] {
  return v
    .replace(/^v/i, '')
    .trim()
    .split(/[-+.]/)
    .map((n) => parseInt(n, 10))
    .filter((n) => !isNaN(n));
}

export function isNewerVersion(remote: string, current: string): boolean {
  const r = parseSemver(remote);
  const c = parseSemver(current);
  const maxLen = Math.max(r.length, c.length);
  for (let i = 0; i < maxLen; i++) {
    const rVal = r[i] ?? 0;
    const cVal = c[i] ?? 0;
    if (rVal > cVal) return true;
    if (rVal < cVal) return false;
  }
  return false;
}

class UpdaterStore {
  updateAvailable = $state<boolean>(false);
  latestVersion = $state<string>('');
  releaseNotes = $state<string>('');
  downloadUrl = $state<string>('');
  publishedAt = $state<string>('');
  status = $state<UpdateStatus>('idle');
  errorMessage = $state<string>('');
  modalOpen = $state<boolean>(false);
  serverOS = $state<string>('');

  openModal() {
    this.modalOpen = true;
  }

  closeModal() {
    this.modalOpen = false;
  }

  async checkForUpdates(silent: boolean = false): Promise<boolean> {
    this.status = 'checking';
    this.errorMessage = '';
    try {
      const healthRes = await fetch('/api/health');
      if (!healthRes.ok) {
        throw new Error(`Health API HTTP ${healthRes.status}`);
      }
      const health = await healthRes.json();
      this.serverOS = typeof health.os === 'string' ? health.os.toLowerCase() : '';

      const res = await fetch('https://api.github.com/repos/m24ih/glorious-web-launcher/releases/latest');
      if (!res.ok) {
        if (res.status === 404) {
          this.status = 'idle';
          return false;
        }
        throw new Error(`GitHub API HTTP ${res.status}`);
      }
      const release = await res.json();
      const rawTag = release.tag_name || '';
      const cleanVer = rawTag.replace(/^v/i, '').trim();
      this.latestVersion = cleanVer;
      this.releaseNotes = release.body || '';
      this.publishedAt = release.published_at ? new Date(release.published_at).toLocaleDateString() : '';

      const assets: any[] = Array.isArray(release.assets) ? release.assets : [];
      const asset = assets.find((a) => {
        if (typeof a.name !== 'string') return false;
        if (this.serverOS === 'linux') {
          return (
            a.name === 'glorious-service-linux-x86_64' ||
            /glorious-launcher.*linux.*\.tar\.gz/i.test(a.name)
          );
        }
        if (this.serverOS === 'windows') {
          return a.name.toLowerCase().endsWith('.exe');
        }
        return false;
      });
      if (asset?.browser_download_url) {
        this.downloadUrl = asset.browser_download_url;
      } else {
        this.downloadUrl = '';
      }

      const newer = isNewerVersion(cleanVer, LAUNCHER_VERSION);
      this.updateAvailable = newer;
      this.status = 'idle';
      return newer;
    } catch (err: any) {
      this.status = silent ? 'idle' : 'error';
      this.errorMessage = err?.message || 'Failed to check for updates';
      return false;
    }
  }

  async installUpdate(): Promise<boolean> {
    if (!this.downloadUrl) {
      this.status = 'error';
      this.errorMessage = 'No download URL available for update installer';
      return false;
    }
    this.status = 'downloading';
    this.errorMessage = '';
    try {
      const res = await fetch('/api/update/install', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          download_url: this.downloadUrl,
          version: this.latestVersion,
        }),
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data.error || `Failed with HTTP ${res.status}`);
      }
      this.status = 'installing';
      return true;
    } catch (err: any) {
      this.status = 'error';
      this.errorMessage = err?.message || 'Failed to install update';
      return false;
    }
  }
}

export const updater = new UpdaterStore();

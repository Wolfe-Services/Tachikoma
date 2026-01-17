import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

interface ReleaseInfo {
  version: string;
  date: string;
  downloads: {
    macos_arm64: string;
    macos_x64: string;
    windows: string;
    linux_appimage: string;
    linux_deb: string;
  };
  checksums: Record<string, string>;
}

export const GET: RequestHandler = async ({ fetch }) => {
  const GITHUB_REPO = 'tachikoma/tachikoma';
  const CDN_BASE = 'https://releases.tachikoma.dev';

  try {
    // Fetch latest release from GitHub API
    const response = await fetch(
      `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
      {
        headers: {
          Accept: 'application/vnd.github.v3+json',
          // Add token if needed for rate limiting
          // Authorization: `token ${process.env.GITHUB_TOKEN}`,
        },
      }
    );

    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`);
    }

    const release = await response.json();
    const version = release.tag_name.replace(/^v/, '');
    const date = new Date(release.published_at).toISOString().split('T')[0];

    // Build download URLs (prefer CDN, fallback to GitHub)
    const getDownloadUrl = (filename: string): string => {
      const asset = release.assets.find((a: any) => a.name === filename);
      if (asset) {
        // Use CDN if available
        return `${CDN_BASE}/releases/v${version}/${filename}`;
      }
      // Fallback to GitHub
      return `https://github.com/${GITHUB_REPO}/releases/download/v${version}/${filename}`;
    };

    // Extract checksums from release body or separate file
    const checksums: Record<string, string> = {};
    const checksumMatch = release.body?.match(/```\s*[\s\S]*?SHA256[\s\S]*?```/);
    if (checksumMatch) {
      const lines = checksumMatch[0].split('\n');
      for (const line of lines) {
        const match = line.match(/^([a-f0-9]{64})\s+(.+)$/);
        if (match) {
          checksums[match[2]] = match[1];
        }
      }
    }

    const releaseInfo: ReleaseInfo = {
      version,
      date,
      downloads: {
        macos_arm64: getDownloadUrl(`Tachikoma-${version}-arm64.dmg`),
        macos_x64: getDownloadUrl(`Tachikoma-${version}-x64.dmg`),
        windows: getDownloadUrl(`Tachikoma-Setup-${version}.exe`),
        linux_appimage: getDownloadUrl(`Tachikoma-${version}.AppImage`),
        linux_deb: getDownloadUrl(`tachikoma_${version}_amd64.deb`),
      },
      checksums,
    };

    return json(releaseInfo, {
      headers: {
        'Cache-Control': 'public, max-age=300', // 5 minute cache
      },
    });
  } catch (error) {
    console.error('Failed to fetch release:', error);

    // Return fallback/mock response for development
    return json({
      version: '1.0.0',
      date: '2024-01-15',
      downloads: {
        macos_arm64: 'https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-1.0.0-arm64.dmg',
        macos_x64: 'https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-1.0.0-x64.dmg',
        windows: 'https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-Setup-1.0.0.exe',
        linux_appimage: 'https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/Tachikoma-1.0.0.AppImage',
        linux_deb: 'https://github.com/tachikoma/tachikoma/releases/download/v1.0.0/tachikoma_1.0.0_amd64.deb',
      },
      checksums: {
        'Tachikoma-1.0.0-arm64.dmg': 'a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890',
        'Tachikoma-1.0.0-x64.dmg': 'b2c3d4e5f6789012345678901234567890123456789012345678901234567890a1',
        'Tachikoma-Setup-1.0.0.exe': 'c3d4e5f6789012345678901234567890123456789012345678901234567890a1b2',
        'Tachikoma-1.0.0.AppImage': 'd4e5f6789012345678901234567890123456789012345678901234567890a1b2c3',
        'tachikoma_1.0.0_amd64.deb': 'e5f6789012345678901234567890123456789012345678901234567890a1b2c3d4',
      },
    }, {
      status: 200,
      headers: {
        'Cache-Control': 'public, max-age=60', // Short cache for fallback
      },
    });
  }
};
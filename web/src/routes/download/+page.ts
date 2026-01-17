import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  try {
    const response = await fetch('/api/releases/latest');
    const release = await response.json();

    return {
      release,
      meta: {
        title: 'Download Tachikoma - AI Development Assistant',
        description: `Download Tachikoma ${release.version} for macOS, Windows, or Linux. AI-powered autonomous development for your desktop.`,
        keywords: 'tachikoma, download, ai assistant, development tools, electron app',
      },
    };
  } catch (error) {
    console.error('Failed to load release data:', error);
    return {
      release: null,
      meta: {
        title: 'Download Tachikoma - AI Development Assistant',
        description: 'Download Tachikoma for macOS, Windows, or Linux. AI-powered autonomous development for your desktop.',
        keywords: 'tachikoma, download, ai assistant, development tools, electron app',
      },
    };
  }
};
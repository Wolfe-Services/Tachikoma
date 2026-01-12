// src/electron/main/windows/taskbar.ts
import { BrowserWindow, nativeImage } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('taskbar');

type ProgressMode = 'none' | 'normal' | 'indeterminate' | 'error' | 'paused';

export function setTaskbarProgress(
  window: BrowserWindow,
  progress: number,
  mode: ProgressMode = 'normal'
): void {
  if (process.platform !== 'win32') {
    logger.debug('Taskbar progress only supported on Windows');
    return;
  }

  try {
    switch (mode) {
      case 'none':
        window.setProgressBar(-1);
        break;
      case 'indeterminate':
        window.setProgressBar(2, { mode: 'indeterminate' });
        break;
      case 'error':
        window.setProgressBar(progress, { mode: 'error' });
        break;
      case 'paused':
        window.setProgressBar(progress, { mode: 'paused' });
        break;
      default:
        window.setProgressBar(progress);
    }
    logger.debug('Taskbar progress updated', { progress, mode });
  } catch (error) {
    logger.error('Failed to set taskbar progress:', error);
  }
}

export function setTaskbarOverlay(
  window: BrowserWindow,
  overlayPath: string | null,
  description: string = ''
): void {
  if (process.platform !== 'win32') {
    logger.debug('Taskbar overlay only supported on Windows');
    return;
  }

  try {
    if (overlayPath) {
      const overlay = nativeImage.createFromPath(overlayPath);
      if (overlay.isEmpty()) {
        logger.warn('Overlay image is empty or invalid', { overlayPath });
        return;
      }
      window.setOverlayIcon(overlay, description);
      logger.debug('Taskbar overlay set', { overlayPath, description });
    } else {
      window.setOverlayIcon(null, '');
      logger.debug('Taskbar overlay cleared');
    }
  } catch (error) {
    logger.error('Failed to set taskbar overlay:', error);
  }
}

export function flashTaskbar(window: BrowserWindow, flash: boolean = true): void {
  if (process.platform !== 'win32') {
    logger.debug('Taskbar flash only supported on Windows');
    return;
  }

  try {
    window.flashFrame(flash);
    logger.debug('Taskbar flash updated', { flash });
  } catch (error) {
    logger.error('Failed to flash taskbar:', error);
  }
}

export function setThumbarButtons(
  window: BrowserWindow,
  buttons: Electron.ThumbarButton[]
): void {
  if (process.platform !== 'win32') {
    logger.debug('Thumbar buttons only supported on Windows');
    return;
  }

  try {
    window.setThumbarButtons(buttons);
    logger.debug('Thumbar buttons set', { buttonCount: buttons.length });
  } catch (error) {
    logger.error('Failed to set thumbar buttons:', error);
  }
}

// Create thumbnail toolbar for media controls
export function setupMediaThumbar(
  window: BrowserWindow,
  callbacks: {
    onPrevious: () => void;
    onPlayPause: () => void;
    onNext: () => void;
  }
): void {
  if (process.platform !== 'win32') {
    return;
  }

  try {
    // Create simple colored icons for media controls if image files don't exist
    const prevIcon = nativeImage.createEmpty();
    const playIcon = nativeImage.createEmpty(); 
    const nextIcon = nativeImage.createEmpty();

    const buttons: Electron.ThumbarButton[] = [
      {
        tooltip: 'Previous',
        icon: prevIcon.isEmpty() ? nativeImage.createFromPath('./build/icons/thumbar-prev.png') : prevIcon,
        click: callbacks.onPrevious,
        flags: ['enabled'],
      },
      {
        tooltip: 'Play/Pause',
        icon: playIcon.isEmpty() ? nativeImage.createFromPath('./build/icons/thumbar-play.png') : playIcon,
        click: callbacks.onPlayPause,
        flags: ['enabled'],
      },
      {
        tooltip: 'Next',
        icon: nextIcon.isEmpty() ? nativeImage.createFromPath('./build/icons/thumbar-next.png') : nextIcon,
        click: callbacks.onNext,
        flags: ['enabled'],
      },
    ];

    setThumbarButtons(window, buttons);
    logger.info('Media thumbar buttons configured');
  } catch (error) {
    logger.error('Failed to setup media thumbar:', error);
  }
}

// Setup development thumbar buttons
export function setupDevelopmentThumbar(
  window: BrowserWindow,
  callbacks: {
    onBuild: () => void;
    onTest: () => void;
    onDebug: () => void;
  }
): void {
  if (process.platform !== 'win32') {
    return;
  }

  try {
    const buildIcon = nativeImage.createEmpty();
    const testIcon = nativeImage.createEmpty();
    const debugIcon = nativeImage.createEmpty();

    const buttons: Electron.ThumbarButton[] = [
      {
        tooltip: 'Build Project',
        icon: buildIcon.isEmpty() ? nativeImage.createFromPath('./build/icons/thumbar-build.png') : buildIcon,
        click: callbacks.onBuild,
        flags: ['enabled'],
      },
      {
        tooltip: 'Run Tests',
        icon: testIcon.isEmpty() ? nativeImage.createFromPath('./build/icons/thumbar-test.png') : testIcon,
        click: callbacks.onTest,
        flags: ['enabled'],
      },
      {
        tooltip: 'Start Debugging',
        icon: debugIcon.isEmpty() ? nativeImage.createFromPath('./build/icons/thumbar-debug.png') : debugIcon,
        click: callbacks.onDebug,
        flags: ['enabled'],
      },
    ];

    setThumbarButtons(window, buttons);
    logger.info('Development thumbar buttons configured');
  } catch (error) {
    logger.error('Failed to setup development thumbar:', error);
  }
}

export function clearThumbarButtons(window: BrowserWindow): void {
  if (process.platform !== 'win32') {
    return;
  }

  try {
    setThumbarButtons(window, []);
    logger.info('Thumbar buttons cleared');
  } catch (error) {
    logger.error('Failed to clear thumbar buttons:', error);
  }
}
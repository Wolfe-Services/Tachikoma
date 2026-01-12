import {
  Menu,
  MenuItemConstructorOptions,
  BrowserWindow,
  clipboard,
  shell,
} from 'electron';
import { ipcMain } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('context-menu');

interface ContextMenuOptions {
  type: 'text' | 'link' | 'image' | 'file' | 'custom';
  selectionText?: string;
  linkURL?: string;
  imageSrc?: string;
  filePath?: string;
  customItems?: MenuItemConstructorOptions[];
  canCut?: boolean;
  canCopy?: boolean;
  canPaste?: boolean;
}

export function setupContextMenuHandlers(): void {
  ipcMain.handle(
    'contextMenu:show',
    (event, options: ContextMenuOptions) => {
      const window = BrowserWindow.fromWebContents(event.sender);
      if (!window) return;

      logger.debug('Showing context menu', { type: options.type });

      const menu = buildContextMenu(options, event.sender);
      menu.popup({ window });
    }
  );

  logger.info('Context menu handlers registered');
}

function buildContextMenu(
  options: ContextMenuOptions,
  webContents: Electron.WebContents
): Menu {
  let template: MenuItemConstructorOptions[] = [];

  switch (options.type) {
    case 'text':
      template = buildTextContextMenu(options, webContents);
      break;
    case 'link':
      template = buildLinkContextMenu(options);
      break;
    case 'image':
      template = buildImageContextMenu(options);
      break;
    case 'file':
      template = buildFileContextMenu(options);
      break;
    case 'custom':
      template = options.customItems || [];
      break;
  }

  return Menu.buildFromTemplate(template);
}

function buildTextContextMenu(
  options: ContextMenuOptions,
  webContents: Electron.WebContents
): MenuItemConstructorOptions[] {
  const hasSelection = !!options.selectionText;

  return [
    {
      label: 'Cut',
      accelerator: 'CmdOrCtrl+X',
      enabled: hasSelection && (options.canCut ?? true),
      click: () => webContents.cut(),
    },
    {
      label: 'Copy',
      accelerator: 'CmdOrCtrl+C',
      enabled: hasSelection && (options.canCopy ?? true),
      click: () => webContents.copy(),
    },
    {
      label: 'Paste',
      accelerator: 'CmdOrCtrl+V',
      enabled: options.canPaste ?? true,
      click: () => webContents.paste(),
    },
    { type: 'separator' },
    {
      label: 'Select All',
      accelerator: 'CmdOrCtrl+A',
      click: () => webContents.selectAll(),
    },
    ...(hasSelection
      ? [
          { type: 'separator' } as MenuItemConstructorOptions,
          {
            label: 'Search Google',
            click: () => {
              const url = `https://www.google.com/search?q=${encodeURIComponent(
                options.selectionText!
              )}`;
              shell.openExternal(url);
            },
          },
        ]
      : []),
  ];
}

function buildLinkContextMenu(
  options: ContextMenuOptions
): MenuItemConstructorOptions[] {
  return [
    {
      label: 'Open Link',
      click: () => shell.openExternal(options.linkURL!),
    },
    {
      label: 'Copy Link Address',
      click: () => clipboard.writeText(options.linkURL!),
    },
    { type: 'separator' },
    {
      label: 'Copy',
      accelerator: 'CmdOrCtrl+C',
      click: () => clipboard.writeText(options.linkURL!),
    },
  ];
}

function buildImageContextMenu(
  options: ContextMenuOptions
): MenuItemConstructorOptions[] {
  return [
    {
      label: 'Copy Image',
      click: () => {
        // Copy image functionality would need to be implemented
        // This would require fetching the image data
        logger.info('Copy image requested', { imageSrc: options.imageSrc });
      },
    },
    {
      label: 'Copy Image Address',
      click: () => clipboard.writeText(options.imageSrc!),
    },
    {
      label: 'Save Image As...',
      click: () => {
        // Save image dialog would need to be implemented
        logger.info('Save image requested', { imageSrc: options.imageSrc });
      },
    },
    { type: 'separator' },
    {
      label: 'Open Image in Browser',
      click: () => shell.openExternal(options.imageSrc!),
    },
  ];
}

function buildFileContextMenu(
  options: ContextMenuOptions
): MenuItemConstructorOptions[] {
  return [
    {
      label: 'Open',
      click: () => shell.openPath(options.filePath!),
    },
    {
      label: 'Reveal in Finder',
      click: () => shell.showItemInFolder(options.filePath!),
    },
    { type: 'separator' },
    {
      label: 'Copy Path',
      click: () => clipboard.writeText(options.filePath!),
    },
    { type: 'separator' },
    {
      label: 'Delete',
      click: () => {
        // Delete confirmation would need to be implemented
        logger.warn('Delete requested but not implemented', { filePath: options.filePath });
      },
    },
  ];
}
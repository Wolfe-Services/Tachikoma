import { dialogService, FileFilter } from './index';

// Common file filters
export const fileFilters: Record<string, FileFilter[]> = {
  images: [
    { name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'] },
  ],
  documents: [
    { name: 'Documents', extensions: ['pdf', 'doc', 'docx', 'txt', 'md'] },
  ],
  code: [
    { name: 'Code', extensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'html', 'css'] },
  ],
  projects: [
    { name: 'Tachikoma Project', extensions: ['tachi', 'tachikoma'] },
  ],
  all: [
    { name: 'All Files', extensions: ['*'] },
  ],
};

export async function openProjectDialog(): Promise<string | null> {
  const paths = await dialogService.showOpenDialog({
    title: 'Open Project',
    directory: true,
  });

  return paths.length > 0 ? paths[0] : null;
}

export async function openFileDialog(filterType?: keyof typeof fileFilters): Promise<string[]> {
  return dialogService.showOpenDialog({
    title: 'Open File',
    filters: filterType ? fileFilters[filterType] : fileFilters.all,
    multiSelect: true,
  });
}

export async function openImageDialog(): Promise<string[]> {
  return dialogService.showOpenDialog({
    title: 'Select Image',
    filters: fileFilters.images,
    multiSelect: true,
  });
}

export async function saveProjectDialog(defaultName?: string): Promise<string | null> {
  return dialogService.showSaveDialog({
    title: 'Save Project',
    defaultName: defaultName || 'untitled.tachi',
    filters: fileFilters.projects,
  });
}

export async function exportDialog(
  defaultName: string,
  filterType: keyof typeof fileFilters
): Promise<string | null> {
  return dialogService.showSaveDialog({
    title: 'Export',
    defaultName,
    filters: fileFilters[filterType],
  });
}

export async function confirmDelete(itemName: string): Promise<boolean> {
  return dialogService.confirm(
    `Delete "${itemName}"?`,
    'This action cannot be undone.'
  );
}

export async function confirmUnsavedChanges(): Promise<'save' | 'discard' | 'cancel'> {
  const result = await dialogService.showMessageBox({
    type: 'warning',
    message: 'You have unsaved changes',
    detail: 'Do you want to save your changes before closing?',
    buttons: ['Save', "Don't Save", 'Cancel'],
    defaultId: 0,
    cancelId: 2,
  });

  switch (result.response) {
    case 0:
      return 'save';
    case 1:
      return 'discard';
    default:
      return 'cancel';
  }
}

export async function confirmOverwrite(filePath: string): Promise<boolean> {
  return dialogService.confirm(
    'File already exists',
    `Do you want to replace "${filePath}"?`
  );
}

export async function showAboutDialog(appInfo: {
  name: string;
  version: string;
  copyright: string;
}): Promise<void> {
  await dialogService.showMessageBox({
    type: 'info',
    title: `About ${appInfo.name}`,
    message: appInfo.name,
    detail: `Version ${appInfo.version}\n\n${appInfo.copyright}`,
  });
}
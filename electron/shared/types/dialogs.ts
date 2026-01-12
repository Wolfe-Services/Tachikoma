export interface DialogAPI {
  // File dialogs
  openFile(options?: OpenFileOptions): Promise<string[]>;
  saveFile(options?: SaveFileOptions): Promise<string | null>;

  // Message dialogs
  message(options: MessageOptions): Promise<MessageResult>;
  confirm(message: string, detail?: string): Promise<boolean>;
  alert(message: string, detail?: string): Promise<void>;
  error(message: string, detail?: string): Promise<void>;

  // Preset dialogs
  openProject(): Promise<string | null>;
  openFiles(filterType?: string): Promise<string[]>;
  saveProject(defaultName?: string): Promise<string | null>;
  confirmUnsavedChanges(): Promise<'save' | 'discard' | 'cancel'>;
  confirmDelete(itemName: string): Promise<boolean>;
  showAbout(appInfo: AppInfo): Promise<void>;

  // Progress dialogs
  progress: {
    create(id: string, options: ProgressOptions): Promise<void>;
    update(id: string, value: number): Promise<void>;
    message(id: string, message: string): Promise<void>;
    close(id: string): Promise<void>;
  };

  // Dialog state management
  hasActive(): Promise<boolean>;
}

export interface OpenFileOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
  multiSelect?: boolean;
  directory?: boolean;
  showHiddenFiles?: boolean;
}

export interface SaveFileOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
  defaultName?: string;
}

export interface MessageOptions {
  type?: 'none' | 'info' | 'error' | 'question' | 'warning';
  title?: string;
  message: string;
  detail?: string;
  buttons?: string[];
  defaultId?: number;
  cancelId?: number;
  checkboxLabel?: string;
  checkboxChecked?: boolean;
}

export interface MessageResult {
  response: number;
  checkboxChecked: boolean;
}

export interface FileFilter {
  name: string;
  extensions: string[];
}

export interface ProgressOptions {
  title: string;
  message?: string;
  indeterminate?: boolean;
}

export interface AppInfo {
  name: string;
  version: string;
  copyright: string;
}
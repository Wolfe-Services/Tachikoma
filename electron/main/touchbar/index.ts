import { TouchBar, BrowserWindow, nativeImage } from 'electron';

const { TouchBarButton, TouchBarSpacer, TouchBarSlider, TouchBarSegmentedControl } = TouchBar;

export function createTouchBar(window: BrowserWindow): TouchBar {
  const newProjectButton = new TouchBarButton({
    label: 'New Project',
    icon: nativeImage.createFromPath('build/touchbar/new.png').resize({ width: 16, height: 16 }),
    click: () => {
      window.webContents.send('menu:newProject');
    },
  });

  const openButton = new TouchBarButton({
    label: 'Open',
    icon: nativeImage.createFromPath('build/touchbar/open.png').resize({ width: 16, height: 16 }),
    click: () => {
      window.webContents.send('menu:open');
    },
  });

  const saveButton = new TouchBarButton({
    label: 'Save',
    icon: nativeImage.createFromPath('build/touchbar/save.png').resize({ width: 16, height: 16 }),
    click: () => {
      window.webContents.send('menu:save');
    },
  });

  const runButton = new TouchBarButton({
    label: 'Run',
    backgroundColor: '#28a745',
    click: () => {
      window.webContents.send('action:run');
    },
  });

  const stopButton = new TouchBarButton({
    label: 'Stop',
    backgroundColor: '#dc3545',
    click: () => {
      window.webContents.send('action:stop');
    },
  });

  const viewSegments = new TouchBarSegmentedControl({
    segments: [
      { label: 'Code' },
      { label: 'Preview' },
      { label: 'Terminal' },
    ],
    selectedIndex: 0,
    change: (selectedIndex) => {
      window.webContents.send('view:change', selectedIndex);
    },
  });

  return new TouchBar({
    items: [
      newProjectButton,
      openButton,
      saveButton,
      new TouchBarSpacer({ size: 'large' }),
      runButton,
      stopButton,
      new TouchBarSpacer({ size: 'flexible' }),
      viewSegments,
    ],
  });
}

export function setupTouchBar(window: BrowserWindow): void {
  if (process.platform !== 'darwin') {
    return;
  }

  const touchBar = createTouchBar(window);
  window.setTouchBar(touchBar);
}

export function updateTouchBarState(window: BrowserWindow, state: {
  isRunning?: boolean;
  currentView?: number;
  canSave?: boolean;
}): void {
  if (process.platform !== 'darwin') {
    return;
  }

  // Create updated TouchBar with current state
  const touchBar = createTouchBar(window);
  
  // Update view segments if provided
  if (state.currentView !== undefined && touchBar.items) {
    const viewControl = touchBar.items.find(item => 
      item instanceof TouchBarSegmentedControl
    ) as TouchBarSegmentedControl;
    
    if (viewControl) {
      viewControl.selectedIndex = state.currentView;
    }
  }
  
  window.setTouchBar(touchBar);
}
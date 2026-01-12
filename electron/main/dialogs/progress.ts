import { BrowserWindow } from 'electron';

interface ProgressOptions {
  title: string;
  message?: string;
  indeterminate?: boolean;
}

export class ProgressDialog {
  private window: BrowserWindow | null = null;
  private parentWindow: BrowserWindow;
  private progress: number = 0;
  private message: string = '';

  constructor(parentWindow: BrowserWindow) {
    this.parentWindow = parentWindow;
  }

  async show(options: ProgressOptions): Promise<void> {
    this.window = new BrowserWindow({
      width: 400,
      height: 120,
      parent: this.parentWindow,
      modal: true,
      frame: false,
      resizable: false,
      movable: false,
      minimizable: false,
      maximizable: false,
      closable: false,
      show: false,
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
      },
    });

    const html = this.generateHTML(options);
    await this.window.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(html)}`);
    this.window.show();

    // Set taskbar progress
    if (options.indeterminate) {
      this.parentWindow.setProgressBar(-1); // Indeterminate
    }
  }

  private generateHTML(options: ProgressOptions): string {
    return `
      <!DOCTYPE html>
      <html>
      <head>
        <style>
          * { margin: 0; padding: 0; box-sizing: border-box; }
          body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a1a;
            color: #fff;
            padding: 20px;
            display: flex;
            flex-direction: column;
            justify-content: center;
            height: 100vh;
          }
          .title {
            font-size: 14px;
            font-weight: 600;
            margin-bottom: 8px;
          }
          .message {
            font-size: 12px;
            color: #888;
            margin-bottom: 12px;
          }
          .progress-container {
            background: #333;
            border-radius: 4px;
            height: 8px;
            overflow: hidden;
          }
          .progress-bar {
            background: #007aff;
            height: 100%;
            width: 0%;
            transition: width 0.3s ease;
          }
          .progress-bar.indeterminate {
            width: 30%;
            animation: indeterminate 1.5s infinite ease-in-out;
          }
          @keyframes indeterminate {
            0% { transform: translateX(-100%); }
            100% { transform: translateX(400%); }
          }
        </style>
      </head>
      <body>
        <div class="title" id="title">${options.title}</div>
        <div class="message" id="message">${options.message || ''}</div>
        <div class="progress-container">
          <div class="progress-bar ${options.indeterminate ? 'indeterminate' : ''}" id="progress"></div>
        </div>
      </body>
      </html>
    `;
  }

  setProgress(value: number): void {
    this.progress = Math.max(0, Math.min(100, value));

    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.executeJavaScript(
        `document.getElementById('progress').style.width = '${this.progress}%'`
      );
    }

    this.parentWindow.setProgressBar(this.progress / 100);
  }

  setMessage(message: string): void {
    this.message = message;

    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.executeJavaScript(
        `document.getElementById('message').textContent = '${message.replace(/'/g, "\\'")}'`
      );
    }
  }

  close(): void {
    if (this.window && !this.window.isDestroyed()) {
      this.window.close();
      this.window = null;
    }

    this.parentWindow.setProgressBar(-1); // Remove progress
  }
}
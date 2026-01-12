import { app } from 'electron';
import { join } from 'path';
import { existsSync, readFileSync, writeFileSync, mkdirSync } from 'fs';

interface AppConfig {
  window: {
    width: number;
    height: number;
    x?: number;
    y?: number;
    maximized: boolean;
  };
  theme: 'light' | 'dark' | 'system';
  locale: string;
  telemetry: boolean;
  autoUpdate: boolean;
  hardwareAcceleration: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
}

const defaultConfig: AppConfig = {
  window: {
    width: 1400,
    height: 900,
    maximized: false,
  },
  theme: 'system',
  locale: 'en',
  telemetry: true,
  autoUpdate: true,
  hardwareAcceleration: true,
  logLevel: 'info',
};

class ConfigManager {
  private configPath: string;
  private config: AppConfig;

  constructor() {
    const userDataPath = app.getPath('userData');
    
    // Ensure userData directory exists
    if (!existsSync(userDataPath)) {
      mkdirSync(userDataPath, { recursive: true });
    }

    this.configPath = join(userDataPath, 'config.json');
    this.config = this.load();
  }

  private load(): AppConfig {
    if (existsSync(this.configPath)) {
      try {
        const data = readFileSync(this.configPath, 'utf-8');
        const parsed = JSON.parse(data);
        
        // Merge with defaults to ensure all properties exist
        return { ...defaultConfig, ...parsed };
      } catch (error) {
        console.error('Failed to load config:', error);
        console.info('Using default configuration');
      }
    }
    
    // Save default config on first run
    const config = { ...defaultConfig };
    this.saveConfig(config);
    return config;
  }

  private saveConfig(config: AppConfig): void {
    try {
      writeFileSync(this.configPath, JSON.stringify(config, null, 2), 'utf-8');
    } catch (error) {
      console.error('Failed to save config:', error);
    }
  }

  save(): void {
    this.saveConfig(this.config);
  }

  get<K extends keyof AppConfig>(key: K): AppConfig[K] {
    return this.config[key];
  }

  set<K extends keyof AppConfig>(key: K, value: AppConfig[K]): void {
    this.config[key] = value;
    this.save();
  }

  getAll(): AppConfig {
    return { ...this.config };
  }

  reset(): void {
    this.config = { ...defaultConfig };
    this.save();
  }

  // Environment-specific overrides
  applyEnvironmentOverrides(): void {
    // Override settings based on environment
    if (process.env.NODE_ENV === 'development') {
      this.config.logLevel = 'debug';
      this.config.telemetry = false;
    }

    if (process.env.NODE_ENV === 'production') {
      this.config.logLevel = 'info';
    }

    // Override from environment variables
    if (process.env.TACHIKOMA_LOG_LEVEL) {
      this.config.logLevel = process.env.TACHIKOMA_LOG_LEVEL as AppConfig['logLevel'];
    }

    if (process.env.TACHIKOMA_DISABLE_TELEMETRY === 'true') {
      this.config.telemetry = false;
    }

    if (process.env.TACHIKOMA_DISABLE_HW_ACCEL === 'true') {
      this.config.hardwareAcceleration = false;
    }
  }
}

const configManager = new ConfigManager();
configManager.applyEnvironmentOverrides();

export { configManager };
export type { AppConfig };
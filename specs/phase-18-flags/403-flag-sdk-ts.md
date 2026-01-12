# 403 - Feature Flag SDK for TypeScript

## Overview

TypeScript/JavaScript SDK for evaluating feature flags in browser and Node.js environments with React hooks support.

## TypeScript Implementation

```typescript
// packages/flags-sdk-ts/src/client.ts

import { EventEmitter } from 'events';

export interface FlagClientConfig {
  apiUrl: string;
  sdkKey: string;
  environment: string;
  streamingEnabled?: boolean;
  cacheTtlMs?: number;
  offlineMode?: boolean;
  timeoutMs?: number;
  onReady?: () => void;
  onError?: (error: Error) => void;
  onFlagChange?: (flagKey: string) => void;
}

export interface Context {
  userId?: string;
  anonymousId?: string;
  groups?: string[];
  properties?: Record<string, unknown>;
}

export interface EvaluationResult<T> {
  value: T;
  reason: string;
  flagKey: string;
  inExperiment?: boolean;
  variant?: string;
}

interface FlagDefinition {
  id: string;
  name: string;
  status: string;
  defaultValue: unknown;
  rules: Rule[];
  rollout?: { percentage: number };
  experiment?: { variants: Variant[] };
  userOverrides: Record<string, unknown>;
  groupOverrides: Record<string, unknown>;
}

interface Rule {
  id: string;
  enabled: boolean;
  conditions: Condition[];
  value: unknown;
}

interface Condition {
  property: string;
  operator: string;
  value: unknown;
}

interface Variant {
  key: string;
  weight: number;
  value: unknown;
}

export class FlagClient extends EventEmitter {
  private config: Required<FlagClientConfig>;
  private flags: Map<string, FlagDefinition> = new Map();
  private evaluationCache: Map<string, { value: unknown; timestamp: number }> = new Map();
  private eventSource?: EventSource;
  private ready = false;
  private readyPromise: Promise<void>;
  private readyResolve!: () => void;

  constructor(config: FlagClientConfig) {
    super();

    this.config = {
      streamingEnabled: true,
      cacheTtlMs: 300000,
      offlineMode: false,
      timeoutMs: 10000,
      onReady: () => {},
      onError: () => {},
      onFlagChange: () => {},
      ...config,
    };

    this.readyPromise = new Promise((resolve) => {
      this.readyResolve = resolve;
    });

    this.initialize();
  }

  private async initialize(): Promise<void> {
    try {
      await this.fetchAllFlags();
      this.ready = true;
      this.readyResolve();
      this.config.onReady?.();
      this.emit('ready');

      if (this.config.streamingEnabled && typeof EventSource !== 'undefined') {
        this.startStreaming();
      }
    } catch (error) {
      this.config.onError?.(error as Error);
      this.emit('error', error);
    }
  }

  private async fetchAllFlags(): Promise<void> {
    const response = await fetch(`${this.config.apiUrl}/sdk/flags`, {
      headers: {
        'Authorization': `Bearer ${this.config.sdkKey}`,
        'X-Environment': this.config.environment,
      },
      signal: AbortSignal.timeout(this.config.timeoutMs),
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch flags: ${response.status}`);
    }

    const flags: FlagDefinition[] = await response.json();
    this.flags.clear();
    for (const flag of flags) {
      this.flags.set(flag.id, flag);
    }
  }

  private startStreaming(): void {
    const url = `${this.config.apiUrl}/sdk/stream?key=${this.config.sdkKey}&env=${this.config.environment}`;
    this.eventSource = new EventSource(url);

    this.eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'flag_update') {
          this.flags.set(data.flag.id, data.flag);
          this.invalidateCache(data.flag.id);
          this.config.onFlagChange?.(data.flag.id);
          this.emit('flagChange', data.flag.id);
        }
      } catch {
        // Ignore parse errors
      }
    };

    this.eventSource.onerror = () => {
      // Reconnect handled automatically by EventSource
    };
  }

  private invalidateCache(flagKey: string): void {
    for (const key of this.evaluationCache.keys()) {
      if (key.startsWith(`${flagKey}:`)) {
        this.evaluationCache.delete(key);
      }
    }
  }

  async waitUntilReady(): Promise<void> {
    return this.readyPromise;
  }

  isReady(): boolean {
    return this.ready;
  }

  getBool(flagKey: string, context: Context, defaultValue: boolean): boolean {
    const result = this.evaluate(flagKey, context);
    return typeof result === 'boolean' ? result : defaultValue;
  }

  getString(flagKey: string, context: Context, defaultValue: string): string {
    const result = this.evaluate(flagKey, context);
    return typeof result === 'string' ? result : defaultValue;
  }

  getNumber(flagKey: string, context: Context, defaultValue: number): number {
    const result = this.evaluate(flagKey, context);
    return typeof result === 'number' ? result : defaultValue;
  }

  getJson<T>(flagKey: string, context: Context, defaultValue: T): T {
    const result = this.evaluate(flagKey, context);
    return result !== undefined ? (result as T) : defaultValue;
  }

  evaluateWithDetails<T>(
    flagKey: string,
    context: Context,
    defaultValue: T
  ): EvaluationResult<T> {
    const flag = this.flags.get(flagKey);

    if (!flag) {
      return {
        value: defaultValue,
        reason: 'not_found',
        flagKey,
      };
    }

    const { value, reason, inExperiment, variant } = this.evaluateFlag(flag, context);

    return {
      value: value !== undefined ? (value as T) : defaultValue,
      reason,
      flagKey,
      inExperiment,
      variant,
    };
  }

  private evaluate(flagKey: string, context: Context): unknown {
    // Check cache
    const cacheKey = this.getCacheKey(flagKey, context);
    const cached = this.evaluationCache.get(cacheKey);
    if (cached && Date.now() - cached.timestamp < this.config.cacheTtlMs) {
      return cached.value;
    }

    const flag = this.flags.get(flagKey);
    if (!flag) {
      return undefined;
    }

    const { value } = this.evaluateFlag(flag, context);

    // Cache result
    this.evaluationCache.set(cacheKey, {
      value,
      timestamp: Date.now(),
    });

    return value;
  }

  private evaluateFlag(
    flag: FlagDefinition,
    context: Context
  ): { value: unknown; reason: string; inExperiment?: boolean; variant?: string } {
    // Check if disabled
    if (flag.status !== 'active') {
      return { value: flag.defaultValue, reason: 'disabled' };
    }

    // Check user overrides
    if (context.userId && flag.userOverrides[context.userId] !== undefined) {
      return {
        value: flag.userOverrides[context.userId],
        reason: 'user_override',
      };
    }

    // Check group overrides
    if (context.groups) {
      for (const group of context.groups) {
        if (flag.groupOverrides[group] !== undefined) {
          return {
            value: flag.groupOverrides[group],
            reason: 'group_override',
          };
        }
      }
    }

    // Evaluate rules
    for (const rule of flag.rules) {
      if (rule.enabled && this.evaluateRule(rule, context)) {
        return { value: rule.value, reason: 'rule_match' };
      }
    }

    // Check rollout
    if (flag.rollout) {
      const key = context.userId || context.anonymousId;
      if (key) {
        const bucket = this.hashToBucket(flag.id, key);
        if (bucket <= flag.rollout.percentage) {
          return {
            value: typeof flag.defaultValue === 'boolean' ? true : flag.defaultValue,
            reason: 'rollout',
          };
        }
      }
    }

    // Check experiment
    if (flag.experiment) {
      const key = context.userId || context.anonymousId;
      if (key) {
        const variant = this.selectVariant(flag.id, key, flag.experiment.variants);
        return {
          value: variant.value,
          reason: 'experiment',
          inExperiment: true,
          variant: variant.key,
        };
      }
    }

    return { value: flag.defaultValue, reason: 'default' };
  }

  private evaluateRule(rule: Rule, context: Context): boolean {
    for (const condition of rule.conditions) {
      const value = context.properties?.[condition.property];
      if (!this.evaluateCondition(condition, value)) {
        return false;
      }
    }
    return true;
  }

  private evaluateCondition(condition: Condition, value: unknown): boolean {
    switch (condition.operator) {
      case 'equals':
        return value === condition.value;
      case 'not_equals':
        return value !== condition.value;
      case 'contains':
        return typeof value === 'string' &&
          typeof condition.value === 'string' &&
          value.includes(condition.value);
      case 'in':
        return Array.isArray(condition.value) && condition.value.includes(value);
      case 'exists':
        return value !== undefined && value !== null;
      case 'not_exists':
        return value === undefined || value === null;
      default:
        return false;
    }
  }

  private hashToBucket(flagKey: string, userKey: string): number {
    const input = `${flagKey}:${userKey}`;
    let hash = 0;
    for (let i = 0; i < input.length; i++) {
      const char = input.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return Math.abs(hash % 100);
  }

  private selectVariant(flagKey: string, userKey: string, variants: Variant[]): Variant {
    const bucket = this.hashToBucket(flagKey, userKey);
    let cumulative = 0;

    for (const variant of variants) {
      cumulative += variant.weight;
      if (bucket <= cumulative) {
        return variant;
      }
    }

    return variants[variants.length - 1];
  }

  private getCacheKey(flagKey: string, context: Context): string {
    const contextKey = context.userId || context.anonymousId || 'anon';
    return `${flagKey}:${contextKey}`;
  }

  async track(flagKey: string, context: Context, value: unknown): Promise<void> {
    try {
      await fetch(`${this.config.apiUrl}/sdk/track`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.config.sdkKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          flagKey,
          userId: context.userId,
          anonymousId: context.anonymousId,
          value,
          timestamp: new Date().toISOString(),
        }),
      });
    } catch {
      // Tracking errors should not throw
    }
  }

  async refresh(): Promise<void> {
    await this.fetchAllFlags();
    this.evaluationCache.clear();
    this.emit('refresh');
  }

  close(): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = undefined;
    }
    this.evaluationCache.clear();
    this.removeAllListeners();
  }
}

// Singleton instance
let instance: FlagClient | null = null;

export function initializeFlags(config: FlagClientConfig): FlagClient {
  if (instance) {
    instance.close();
  }
  instance = new FlagClient(config);
  return instance;
}

export function getFlags(): FlagClient {
  if (!instance) {
    throw new Error('FlagClient not initialized. Call initializeFlags first.');
  }
  return instance;
}
```

## React Hooks

```typescript
// packages/flags-sdk-ts/src/react.tsx

import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  useMemo,
  ReactNode,
} from 'react';
import { FlagClient, FlagClientConfig, Context, EvaluationResult } from './client';

interface FlagContextValue {
  client: FlagClient | null;
  isReady: boolean;
  context: Context;
  setContext: (context: Context) => void;
}

const FlagContext = createContext<FlagContextValue | null>(null);

export interface FlagProviderProps {
  config: FlagClientConfig;
  context?: Context;
  children: ReactNode;
}

export function FlagProvider({ config, context: initialContext, children }: FlagProviderProps) {
  const [client, setClient] = useState<FlagClient | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [context, setContext] = useState<Context>(initialContext || {});

  useEffect(() => {
    const flagClient = new FlagClient({
      ...config,
      onReady: () => {
        setIsReady(true);
        config.onReady?.();
      },
      onError: config.onError,
    });

    setClient(flagClient);

    return () => {
      flagClient.close();
    };
  }, [config.apiUrl, config.sdkKey, config.environment]);

  const value = useMemo(
    () => ({ client, isReady, context, setContext }),
    [client, isReady, context]
  );

  return (
    <FlagContext.Provider value={value}>
      {children}
    </FlagContext.Provider>
  );
}

function useFlagContext(): FlagContextValue {
  const context = useContext(FlagContext);
  if (!context) {
    throw new Error('useFlagContext must be used within a FlagProvider');
  }
  return context;
}

export function useIsReady(): boolean {
  const { isReady } = useFlagContext();
  return isReady;
}

export function useFlagContext$(): Context {
  const { context } = useFlagContext();
  return context;
}

export function useSetFlagContext(): (context: Context) => void {
  const { setContext } = useFlagContext();
  return setContext;
}

export function useBoolFlag(flagKey: string, defaultValue: boolean): boolean {
  const { client, isReady, context } = useFlagContext();
  const [value, setValue] = useState(defaultValue);

  useEffect(() => {
    if (!client || !isReady) {
      setValue(defaultValue);
      return;
    }

    setValue(client.getBool(flagKey, context, defaultValue));

    const handleChange = (changedKey: string) => {
      if (changedKey === flagKey) {
        setValue(client.getBool(flagKey, context, defaultValue));
      }
    };

    client.on('flagChange', handleChange);
    return () => {
      client.off('flagChange', handleChange);
    };
  }, [client, isReady, flagKey, context, defaultValue]);

  return value;
}

export function useStringFlag(flagKey: string, defaultValue: string): string {
  const { client, isReady, context } = useFlagContext();
  const [value, setValue] = useState(defaultValue);

  useEffect(() => {
    if (!client || !isReady) {
      setValue(defaultValue);
      return;
    }

    setValue(client.getString(flagKey, context, defaultValue));

    const handleChange = (changedKey: string) => {
      if (changedKey === flagKey) {
        setValue(client.getString(flagKey, context, defaultValue));
      }
    };

    client.on('flagChange', handleChange);
    return () => {
      client.off('flagChange', handleChange);
    };
  }, [client, isReady, flagKey, context, defaultValue]);

  return value;
}

export function useNumberFlag(flagKey: string, defaultValue: number): number {
  const { client, isReady, context } = useFlagContext();
  const [value, setValue] = useState(defaultValue);

  useEffect(() => {
    if (!client || !isReady) {
      setValue(defaultValue);
      return;
    }

    setValue(client.getNumber(flagKey, context, defaultValue));

    const handleChange = (changedKey: string) => {
      if (changedKey === flagKey) {
        setValue(client.getNumber(flagKey, context, defaultValue));
      }
    };

    client.on('flagChange', handleChange);
    return () => {
      client.off('flagChange', handleChange);
    };
  }, [client, isReady, flagKey, context, defaultValue]);

  return value;
}

export function useJsonFlag<T>(flagKey: string, defaultValue: T): T {
  const { client, isReady, context } = useFlagContext();
  const [value, setValue] = useState<T>(defaultValue);

  useEffect(() => {
    if (!client || !isReady) {
      setValue(defaultValue);
      return;
    }

    setValue(client.getJson(flagKey, context, defaultValue));

    const handleChange = (changedKey: string) => {
      if (changedKey === flagKey) {
        setValue(client.getJson(flagKey, context, defaultValue));
      }
    };

    client.on('flagChange', handleChange);
    return () => {
      client.off('flagChange', handleChange);
    };
  }, [client, isReady, flagKey, context, defaultValue]);

  return value;
}

export function useFlagDetails<T>(
  flagKey: string,
  defaultValue: T
): EvaluationResult<T> {
  const { client, isReady, context } = useFlagContext();
  const [result, setResult] = useState<EvaluationResult<T>>({
    value: defaultValue,
    reason: 'not_ready',
    flagKey,
  });

  useEffect(() => {
    if (!client || !isReady) {
      setResult({
        value: defaultValue,
        reason: 'not_ready',
        flagKey,
      });
      return;
    }

    setResult(client.evaluateWithDetails(flagKey, context, defaultValue));

    const handleChange = (changedKey: string) => {
      if (changedKey === flagKey) {
        setResult(client.evaluateWithDetails(flagKey, context, defaultValue));
      }
    };

    client.on('flagChange', handleChange);
    return () => {
      client.off('flagChange', handleChange);
    };
  }, [client, isReady, flagKey, context, defaultValue]);

  return result;
}

export function useExperiment(
  experimentKey: string
): { variant: string; inExperiment: boolean } {
  const result = useFlagDetails(experimentKey, 'control');

  return {
    variant: result.variant || (result.value as string),
    inExperiment: result.inExperiment || false,
  };
}

export function useTrackExposure(): (flagKey: string) => void {
  const { client, context } = useFlagContext();

  return useCallback(
    (flagKey: string) => {
      if (client) {
        const value = client.getJson(flagKey, context, null);
        client.track(flagKey, context, value);
      }
    },
    [client, context]
  );
}

// Feature component for conditional rendering
interface FeatureProps {
  flag: string;
  children: ReactNode;
  fallback?: ReactNode;
}

export function Feature({ flag, children, fallback = null }: FeatureProps) {
  const enabled = useBoolFlag(flag, false);
  return <>{enabled ? children : fallback}</>;
}

// Variant component for A/B tests
interface VariantProps {
  experiment: string;
  variant: string;
  children: ReactNode;
}

export function Variant({ experiment, variant, children }: VariantProps) {
  const { variant: selectedVariant } = useExperiment(experiment);
  return <>{selectedVariant === variant ? children : null}</>;
}
```

## Usage Example

```tsx
import { FlagProvider, useBoolFlag, useExperiment, Feature, Variant } from '@tachikoma/flags-sdk';

// App setup
function App() {
  return (
    <FlagProvider
      config={{
        apiUrl: 'https://flags.example.com',
        sdkKey: process.env.NEXT_PUBLIC_FLAGS_SDK_KEY!,
        environment: 'production',
      }}
      context={{ userId: 'user-123' }}
    >
      <MyComponent />
    </FlagProvider>
  );
}

// Using hooks
function MyComponent() {
  const newDashboard = useBoolFlag('new-dashboard', false);
  const { variant, inExperiment } = useExperiment('checkout-flow');

  return (
    <div>
      {newDashboard ? <NewDashboard /> : <OldDashboard />}

      {inExperiment && (
        <p>You are in the {variant} group</p>
      )}
    </div>
  );
}

// Using components
function FeatureExample() {
  return (
    <Feature flag="premium-features" fallback={<UpgradeBanner />}>
      <PremiumContent />
    </Feature>
  );
}

function ABTestExample() {
  return (
    <>
      <Variant experiment="hero-design" variant="control">
        <HeroV1 />
      </Variant>
      <Variant experiment="hero-design" variant="treatment">
        <HeroV2 />
      </Variant>
    </>
  );
}
```

## Related Specs

- 402-flag-sdk-rust.md - Rust SDK
- 394-flag-evaluation.md - Evaluation logic
- 404-flag-sync.md - Synchronization

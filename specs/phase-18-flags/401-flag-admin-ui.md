# 401 - Feature Flag Admin UI

## Overview

Administrative interface for managing feature flags, including CRUD operations, targeting configuration, and monitoring.


## Acceptance Criteria
- [x] Implementation complete per spec

## React Components

```typescript
// packages/flags-admin/src/components/FlagList.tsx

import React, { useState, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Table,
  TableHead,
  TableBody,
  TableRow,
  TableCell,
  Switch,
  Chip,
  IconButton,
  TextField,
  Select,
  MenuItem,
  Button,
} from '@/components/ui';
import { FlagDefinition, FlagStatus } from '../types';
import { flagsApi } from '../api';

interface FlagListProps {
  onSelect: (flag: FlagDefinition) => void;
  onCreate: () => void;
}

export function FlagList({ onSelect, onCreate }: FlagListProps) {
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<FlagStatus | 'all'>('all');
  const [tagFilter, setTagFilter] = useState<string | 'all'>('all');

  const queryClient = useQueryClient();

  const { data: flags, isLoading } = useQuery({
    queryKey: ['flags', statusFilter, tagFilter],
    queryFn: () => flagsApi.listFlags({
      status: statusFilter === 'all' ? undefined : statusFilter,
      tags: tagFilter === 'all' ? undefined : [tagFilter],
    }),
  });

  const toggleMutation = useMutation({
    mutationFn: ({ flagId, enabled }: { flagId: string; enabled: boolean }) =>
      flagsApi.toggleFlag(flagId, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['flags'] });
    },
  });

  const filteredFlags = useMemo(() => {
    if (!flags) return [];
    return flags.filter(flag =>
      flag.name.toLowerCase().includes(search.toLowerCase()) ||
      flag.id.toLowerCase().includes(search.toLowerCase())
    );
  }, [flags, search]);

  const allTags = useMemo(() => {
    if (!flags) return [];
    const tagSet = new Set<string>();
    flags.forEach(flag => flag.metadata.tags.forEach(tag => tagSet.add(tag)));
    return Array.from(tagSet);
  }, [flags]);

  if (isLoading) {
    return <div>Loading flags...</div>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Feature Flags</h1>
        <Button onClick={onCreate}>Create Flag</Button>
      </div>

      <div className="flex gap-4">
        <TextField
          placeholder="Search flags..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="flex-1"
        />
        <Select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value as any)}>
          <MenuItem value="all">All Statuses</MenuItem>
          <MenuItem value="active">Active</MenuItem>
          <MenuItem value="disabled">Disabled</MenuItem>
          <MenuItem value="deprecated">Deprecated</MenuItem>
        </Select>
        <Select value={tagFilter} onChange={(e) => setTagFilter(e.target.value)}>
          <MenuItem value="all">All Tags</MenuItem>
          {allTags.map(tag => (
            <MenuItem key={tag} value={tag}>{tag}</MenuItem>
          ))}
        </Select>
      </div>

      <Table>
        <TableHead>
          <TableRow>
            <TableCell>Name</TableCell>
            <TableCell>Key</TableCell>
            <TableCell>Status</TableCell>
            <TableCell>Type</TableCell>
            <TableCell>Tags</TableCell>
            <TableCell>Enabled</TableCell>
            <TableCell>Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {filteredFlags.map(flag => (
            <TableRow key={flag.id} onClick={() => onSelect(flag)} className="cursor-pointer hover:bg-gray-50">
              <TableCell className="font-medium">{flag.name}</TableCell>
              <TableCell className="font-mono text-sm">{flag.id}</TableCell>
              <TableCell>
                <StatusBadge status={flag.status} />
              </TableCell>
              <TableCell>{flag.valueType}</TableCell>
              <TableCell>
                <div className="flex gap-1 flex-wrap">
                  {flag.metadata.tags.map(tag => (
                    <Chip key={tag} size="small">{tag}</Chip>
                  ))}
                </div>
              </TableCell>
              <TableCell onClick={(e) => e.stopPropagation()}>
                <Switch
                  checked={flag.status === 'active'}
                  onChange={(checked) => toggleMutation.mutate({
                    flagId: flag.id,
                    enabled: checked,
                  })}
                />
              </TableCell>
              <TableCell>
                <IconButton onClick={(e) => {
                  e.stopPropagation();
                  onSelect(flag);
                }}>
                  Edit
                </IconButton>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

function StatusBadge({ status }: { status: FlagStatus }) {
  const colors: Record<FlagStatus, string> = {
    active: 'bg-green-100 text-green-800',
    disabled: 'bg-gray-100 text-gray-800',
    testing: 'bg-blue-100 text-blue-800',
    deprecated: 'bg-yellow-100 text-yellow-800',
    archived: 'bg-red-100 text-red-800',
  };

  return (
    <span className={`px-2 py-1 rounded-full text-xs font-medium ${colors[status]}`}>
      {status}
    </span>
  );
}
```

```typescript
// packages/flags-admin/src/components/FlagEditor.tsx

import React, { useState } from 'react';
import { useForm, Controller } from 'react-hook-form';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import {
  TextField,
  Select,
  MenuItem,
  Button,
  Switch,
  Tabs,
  Tab,
  TabPanel,
} from '@/components/ui';
import { FlagDefinition, FlagValueType, Environment } from '../types';
import { flagsApi } from '../api';
import { RuleEditor } from './RuleEditor';
import { RolloutEditor } from './RolloutEditor';
import { TargetingEditor } from './TargetingEditor';
import { OverrideEditor } from './OverrideEditor';

interface FlagEditorProps {
  flag?: FlagDefinition;
  onSave: () => void;
  onCancel: () => void;
}

export function FlagEditor({ flag, onSave, onCancel }: FlagEditorProps) {
  const [activeTab, setActiveTab] = useState(0);
  const queryClient = useQueryClient();

  const { control, handleSubmit, watch, setValue } = useForm<FlagDefinition>({
    defaultValues: flag || {
      id: '',
      name: '',
      description: '',
      status: 'disabled',
      valueType: 'boolean',
      defaultValue: { type: 'boolean', value: false },
      environments: [],
      rules: [],
      rollout: null,
      experiment: null,
      userOverrides: {},
      groupOverrides: {},
      metadata: {
        createdAt: new Date(),
        createdBy: '',
        updatedAt: new Date(),
        updatedBy: '',
        tags: [],
        owner: '',
      },
    },
  });

  const saveMutation = useMutation({
    mutationFn: (data: FlagDefinition) =>
      flag ? flagsApi.updateFlag(flag.id, data) : flagsApi.createFlag(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['flags'] });
      onSave();
    },
  });

  const valueType = watch('valueType');

  return (
    <form onSubmit={handleSubmit((data) => saveMutation.mutate(data))} className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold">
          {flag ? 'Edit Flag' : 'Create Flag'}
        </h2>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onCancel}>Cancel</Button>
          <Button type="submit" loading={saveMutation.isPending}>Save</Button>
        </div>
      </div>

      <Tabs value={activeTab} onChange={setActiveTab}>
        <Tab label="Basic" />
        <Tab label="Targeting" />
        <Tab label="Rollout" />
        <Tab label="Overrides" />
        <Tab label="Settings" />
      </Tabs>

      <TabPanel value={activeTab} index={0}>
        <div className="space-y-4">
          <Controller
            name="id"
            control={control}
            rules={{ required: true, pattern: /^[a-z0-9-]+$/ }}
            render={({ field, fieldState }) => (
              <TextField
                {...field}
                label="Flag Key"
                placeholder="my-feature-flag"
                disabled={!!flag}
                error={fieldState.error?.message}
                helperText="Lowercase letters, numbers, and hyphens only"
              />
            )}
          />

          <Controller
            name="name"
            control={control}
            rules={{ required: true }}
            render={({ field, fieldState }) => (
              <TextField
                {...field}
                label="Name"
                placeholder="My Feature Flag"
                error={fieldState.error?.message}
              />
            )}
          />

          <Controller
            name="description"
            control={control}
            render={({ field }) => (
              <TextField
                {...field}
                label="Description"
                multiline
                rows={3}
                placeholder="Describe what this flag controls..."
              />
            )}
          />

          <Controller
            name="valueType"
            control={control}
            render={({ field }) => (
              <Select {...field} label="Value Type">
                <MenuItem value="boolean">Boolean (on/off)</MenuItem>
                <MenuItem value="string">String</MenuItem>
                <MenuItem value="number">Number</MenuItem>
                <MenuItem value="json">JSON</MenuItem>
                <MenuItem value="variant">Variant (A/B Test)</MenuItem>
              </Select>
            )}
          />

          <DefaultValueEditor
            valueType={valueType}
            control={control}
          />

          <Controller
            name="status"
            control={control}
            render={({ field }) => (
              <Select {...field} label="Status">
                <MenuItem value="disabled">Disabled</MenuItem>
                <MenuItem value="active">Active</MenuItem>
                <MenuItem value="testing">Testing</MenuItem>
                <MenuItem value="deprecated">Deprecated</MenuItem>
              </Select>
            )}
          />
        </div>
      </TabPanel>

      <TabPanel value={activeTab} index={1}>
        <TargetingEditor control={control} />
      </TabPanel>

      <TabPanel value={activeTab} index={2}>
        <RolloutEditor control={control} valueType={valueType} />
      </TabPanel>

      <TabPanel value={activeTab} index={3}>
        <OverrideEditor control={control} flagId={flag?.id} />
      </TabPanel>

      <TabPanel value={activeTab} index={4}>
        <SettingsEditor control={control} flag={flag} />
      </TabPanel>
    </form>
  );
}

function DefaultValueEditor({ valueType, control }: { valueType: FlagValueType; control: any }) {
  switch (valueType) {
    case 'boolean':
      return (
        <Controller
          name="defaultValue.value"
          control={control}
          render={({ field }) => (
            <div className="flex items-center gap-2">
              <label>Default Value:</label>
              <Switch {...field} checked={field.value} />
              <span>{field.value ? 'Enabled' : 'Disabled'}</span>
            </div>
          )}
        />
      );

    case 'string':
      return (
        <Controller
          name="defaultValue.value"
          control={control}
          render={({ field }) => (
            <TextField {...field} label="Default Value" />
          )}
        />
      );

    case 'number':
      return (
        <Controller
          name="defaultValue.value"
          control={control}
          render={({ field }) => (
            <TextField {...field} label="Default Value" type="number" />
          )}
        />
      );

    case 'json':
      return (
        <Controller
          name="defaultValue.value"
          control={control}
          render={({ field }) => (
            <TextField
              {...field}
              label="Default Value (JSON)"
              multiline
              rows={4}
              value={typeof field.value === 'string' ? field.value : JSON.stringify(field.value, null, 2)}
              onChange={(e) => {
                try {
                  field.onChange(JSON.parse(e.target.value));
                } catch {
                  field.onChange(e.target.value);
                }
              }}
            />
          )}
        />
      );

    default:
      return null;
  }
}

function SettingsEditor({ control, flag }: { control: any; flag?: FlagDefinition }) {
  return (
    <div className="space-y-4">
      <Controller
        name="metadata.tags"
        control={control}
        render={({ field }) => (
          <TextField
            label="Tags"
            placeholder="frontend, experiment, beta"
            value={field.value.join(', ')}
            onChange={(e) => field.onChange(
              e.target.value.split(',').map(t => t.trim()).filter(Boolean)
            )}
            helperText="Comma-separated list of tags"
          />
        )}
      />

      <Controller
        name="metadata.owner"
        control={control}
        render={({ field }) => (
          <TextField
            {...field}
            label="Owner"
            placeholder="team-name or user@example.com"
          />
        )}
      />

      <Controller
        name="metadata.documentationUrl"
        control={control}
        render={({ field }) => (
          <TextField
            {...field}
            label="Documentation URL"
            placeholder="https://..."
          />
        )}
      />

      {flag && (
        <div className="pt-4 border-t space-y-2 text-sm text-gray-600">
          <p>Created: {new Date(flag.metadata.createdAt).toLocaleString()} by {flag.metadata.createdBy}</p>
          <p>Updated: {new Date(flag.metadata.updatedAt).toLocaleString()} by {flag.metadata.updatedBy}</p>
        </div>
      )}
    </div>
  );
}
```

```typescript
// packages/flags-admin/src/components/FlagDashboard.tsx

import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardHeader, CardContent } from '@/components/ui';
import { BarChart, LineChart, PieChart } from '@/components/charts';
import { flagsApi } from '../api';

export function FlagDashboard() {
  const { data: stats } = useQuery({
    queryKey: ['flag-stats'],
    queryFn: flagsApi.getStats,
  });

  const { data: evaluations } = useQuery({
    queryKey: ['flag-evaluations'],
    queryFn: () => flagsApi.getEvaluationStats({ period: '24h' }),
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Feature Flags Dashboard</h1>

      <div className="grid grid-cols-4 gap-4">
        <StatCard
          title="Total Flags"
          value={stats?.totalFlags || 0}
          change={stats?.flagsCreatedThisWeek}
        />
        <StatCard
          title="Active Flags"
          value={stats?.activeFlags || 0}
        />
        <StatCard
          title="Evaluations (24h)"
          value={stats?.evaluationsLast24h || 0}
          format="compact"
        />
        <StatCard
          title="Avg Eval Time"
          value={stats?.avgEvaluationTimeUs || 0}
          suffix="μs"
        />
      </div>

      <div className="grid grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <h3 className="font-semibold">Evaluations Over Time</h3>
          </CardHeader>
          <CardContent>
            <LineChart
              data={evaluations?.timeline || []}
              xKey="timestamp"
              yKey="count"
              height={300}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <h3 className="font-semibold">Flag Status Distribution</h3>
          </CardHeader>
          <CardContent>
            <PieChart
              data={[
                { name: 'Active', value: stats?.activeFlags || 0 },
                { name: 'Disabled', value: stats?.disabledFlags || 0 },
                { name: 'Testing', value: stats?.testingFlags || 0 },
                { name: 'Deprecated', value: stats?.deprecatedFlags || 0 },
              ]}
              height={300}
            />
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <h3 className="font-semibold">Top Evaluated Flags</h3>
        </CardHeader>
        <CardContent>
          <BarChart
            data={stats?.topFlags || []}
            xKey="flagId"
            yKey="evaluations"
            height={300}
          />
        </CardContent>
      </Card>
    </div>
  );
}

function StatCard({
  title,
  value,
  change,
  suffix,
  format,
}: {
  title: string;
  value: number;
  change?: number;
  suffix?: string;
  format?: 'compact' | 'default';
}) {
  const formattedValue = format === 'compact'
    ? new Intl.NumberFormat('en', { notation: 'compact' }).format(value)
    : value.toLocaleString();

  return (
    <Card>
      <CardContent className="p-4">
        <p className="text-sm text-gray-500">{title}</p>
        <p className="text-2xl font-bold">
          {formattedValue}{suffix}
        </p>
        {change !== undefined && (
          <p className={`text-sm ${change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
            {change >= 0 ? '+' : ''}{change} this week
          </p>
        )}
      </CardContent>
    </Card>
  );
}
```

## API Routes

```typescript
// packages/flags-admin/src/api/routes.ts

import { Router } from 'express';
import { flagService } from '../services/flagService';
import { validateRequest, requireAuth, requireRole } from '../middleware';
import { z } from 'zod';

const router = Router();

// List flags
router.get('/flags', requireAuth, async (req, res) => {
  const { status, tags, owner, limit, offset } = req.query;

  const flags = await flagService.listFlags({
    status: status as any,
    tags: tags ? (tags as string).split(',') : undefined,
    owner: owner as string,
    limit: parseInt(limit as string) || 50,
    offset: parseInt(offset as string) || 0,
  });

  res.json(flags);
});

// Get single flag
router.get('/flags/:id', requireAuth, async (req, res) => {
  const flag = await flagService.getFlag(req.params.id);
  if (!flag) {
    return res.status(404).json({ error: 'Flag not found' });
  }
  res.json(flag);
});

// Create flag
router.post('/flags', requireAuth, requireRole('admin'), async (req, res) => {
  const flag = await flagService.createFlag(req.body, req.user.id);
  res.status(201).json(flag);
});

// Update flag
router.put('/flags/:id', requireAuth, requireRole('admin'), async (req, res) => {
  const flag = await flagService.updateFlag(req.params.id, req.body, req.user.id);
  res.json(flag);
});

// Toggle flag
router.post('/flags/:id/toggle', requireAuth, requireRole('admin'), async (req, res) => {
  const { enabled } = req.body;
  const flag = await flagService.toggleFlag(req.params.id, enabled, req.user.id);
  res.json(flag);
});

// Delete flag
router.delete('/flags/:id', requireAuth, requireRole('admin'), async (req, res) => {
  await flagService.deleteFlag(req.params.id, req.user.id);
  res.status(204).send();
});

// Get flag statistics
router.get('/flags/:id/stats', requireAuth, async (req, res) => {
  const stats = await flagService.getFlagStats(req.params.id);
  res.json(stats);
});

// Get dashboard stats
router.get('/stats', requireAuth, async (req, res) => {
  const stats = await flagService.getDashboardStats();
  res.json(stats);
});

export { router as flagsRouter };
```

## Related Specs

- 392-flag-definition.md - Flag structure
- 409-flag-api.md - Full API specification
- 407-flag-audit.md - Audit logging

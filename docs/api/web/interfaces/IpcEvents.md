[**tachikoma-web v0.1.0**](../README.md)

***

[tachikoma-web](../README.md) / IpcEvents

# Interface: IpcEvents

Defined in: [ipc/types.ts:41](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L41)

## Properties

### mission:progress

> **mission:progress**: `object`

Defined in: [ipc/types.ts:42](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L42)

#### missionId

> **missionId**: `string`

#### progress

> **progress**: `number`

#### message

> **message**: `string`

***

### mission:log

> **mission:log**: `object`

Defined in: [ipc/types.ts:43](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L43)

#### missionId

> **missionId**: `string`

#### level

> **level**: `"info"` \| `"warn"` \| `"error"`

#### message

> **message**: `string`

***

### mission:complete

> **mission:complete**: `object`

Defined in: [ipc/types.ts:44](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L44)

#### missionId

> **missionId**: `string`

#### success

> **success**: `boolean`

#### summary

> **summary**: `string`

***

### mission:error

> **mission:error**: `object`

Defined in: [ipc/types.ts:45](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L45)

#### missionId

> **missionId**: `string`

#### error

> **error**: `string`

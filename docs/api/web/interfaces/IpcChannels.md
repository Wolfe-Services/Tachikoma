[**tachikoma-web v0.1.0**](../README.md)

***

[tachikoma-web](../README.md) / IpcChannels

# Interface: IpcChannels

Defined in: [ipc/types.ts:4](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L4)

## Properties

### mission:start

> **mission:start**: `object`

Defined in: [ipc/types.ts:6](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L6)

#### request

> **request**: `object`

##### request.specPath

> **specPath**: `string`

##### request.backend

> **backend**: `string`

##### request.mode

> **mode**: `"attended"` \| `"unattended"`

#### response

> **response**: `object`

##### response.missionId

> **missionId**: `string`

***

### mission:stop

> **mission:stop**: `object`

Defined in: [ipc/types.ts:10](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L10)

#### request

> **request**: `object`

##### request.missionId

> **missionId**: `string`

#### response

> **response**: `object`

##### response.success

> **success**: `boolean`

***

### mission:status

> **mission:status**: `object`

Defined in: [ipc/types.ts:14](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L14)

#### request

> **request**: `object`

##### request.missionId

> **missionId**: `string`

#### response

> **response**: [`MissionStatus`](MissionStatus.md)

***

### spec:list

> **spec:list**: `object`

Defined in: [ipc/types.ts:20](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L20)

#### request

> **request**: `object`

##### request.path?

> `optional` **path**: `string`

#### response

> **response**: [`SpecFile`](SpecFile.md)[]

***

### spec:read

> **spec:read**: `object`

Defined in: [ipc/types.ts:24](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L24)

#### request

> **request**: `object`

##### request.path

> **path**: `string`

#### response

> **response**: `object`

##### response.content

> **content**: `string`

##### response.metadata

> **metadata**: [`SpecMetadata`](SpecMetadata.md)

***

### config:get

> **config:get**: `object`

Defined in: [ipc/types.ts:30](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L30)

#### request

> **request**: `object`

##### request.key?

> `optional` **key**: `string`

#### response

> **response**: [`TachikomaConfig`](TachikomaConfig.md)

***

### config:set

> **config:set**: `object`

Defined in: [ipc/types.ts:34](https://github.com/DwGitmo/Tachikoma/blob/a721b2776b07d9315755397b2c957e097f38c1b0/web/src/lib/ipc/types.ts#L34)

#### request

> **request**: `object`

##### request.key

> **key**: `string`

##### request.value

> **value**: `unknown`

#### response

> **response**: `object`

##### response.success

> **success**: `boolean`

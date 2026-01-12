[**tachikoma-web v0.1.0**](../README.md)

***

[tachikoma-web](../README.md) / missionStore

# Variable: missionStore

> `const` **missionStore**: `object`

Defined in: [stores/mission.ts:57](https://github.com/DwGitmo/Tachikoma/blob/e91ef3d0907335fcbe422629c419d70314ca9f13/web/src/lib/stores/mission.ts#L57)

## Type Declaration

### subscribe()

> **subscribe**: (`this`, `run`, `invalidate?`) => `Unsubscriber`

Subscribe on value changes.

#### Parameters

##### this

`void`

##### run

`Subscriber`\<`MissionState`\>

subscription callback

##### invalidate?

`Invalidator`\<`MissionState`\>

cleanup callback

#### Returns

`Unsubscriber`

### start()

> **start**(`specPath`, `backend`, `mode`): `Promise`\<`string`\>

#### Parameters

##### specPath

`string`

##### backend

`string`

##### mode

`"attended"` | `"unattended"`

#### Returns

`Promise`\<`string`\>

### stop()

> **stop**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

### addLog()

> **addLog**(`level`, `message`): `void`

#### Parameters

##### level

`string`

##### message

`string`

#### Returns

`void`

### clear()

> **clear**(): `void`

#### Returns

`void`

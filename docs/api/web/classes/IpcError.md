[**tachikoma-web v0.1.0**](../README.md)

***

[tachikoma-web](../README.md) / IpcError

# Class: IpcError

Defined in: [ipc/errors.ts:1](https://github.com/DwGitmo/Tachikoma/blob/e91ef3d0907335fcbe422629c419d70314ca9f13/web/src/lib/ipc/errors.ts#L1)

## Extends

- `Error`

## Constructors

### Constructor

> **new IpcError**(`message`, `channel`, `originalError?`): `IpcError`

Defined in: [ipc/errors.ts:2](https://github.com/DwGitmo/Tachikoma/blob/e91ef3d0907335fcbe422629c419d70314ca9f13/web/src/lib/ipc/errors.ts#L2)

#### Parameters

##### message

`string`

##### channel

`string`

##### originalError?

`unknown`

#### Returns

`IpcError`

#### Overrides

`Error.constructor`

## Properties

### channel

> **channel**: `string`

Defined in: [ipc/errors.ts:4](https://github.com/DwGitmo/Tachikoma/blob/e91ef3d0907335fcbe422629c419d70314ca9f13/web/src/lib/ipc/errors.ts#L4)

***

### originalError?

> `optional` **originalError**: `unknown`

Defined in: [ipc/errors.ts:5](https://github.com/DwGitmo/Tachikoma/blob/e91ef3d0907335fcbe422629c419d70314ca9f13/web/src/lib/ipc/errors.ts#L5)

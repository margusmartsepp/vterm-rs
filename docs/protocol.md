# Wire protocol

Status: **unstable** — will be pinned at v1.0. Until then, breaking changes are documented
in [`CHANGELOG.md`](../CHANGELOG.md).

## Transport

- Windows named pipe: `\\.\pipe\vterm-rs-skill`.
- One JSON object per line. UTF-8. Trailing `\r\n` or `\n` both accepted.
- Bidirectional. Server is singleton; only one client connection at a time (today).

## Request envelope

```json
{
  "req_id": 7,
  "type": "Spawn",
  "payload": { "title": "build", "visible": false }
}
```

| Field     | Type    | Required | Notes                                                |
| --------- | ------- | -------- | ---------------------------------------------------- |
| `req_id`  | u64     | no       | Echoed verbatim on the response                      |
| `type`    | string  | yes      | One of the variants below                            |
| `payload` | object  | varies   | Per-variant; see below                               |

## Response envelope

```json
{
  "req_id": 7,
  "status": "success",
  "duration_ms": 11,
  "id": 1
}
```

| Field          | Type     | Notes                                                       |
| -------------- | -------- | ----------------------------------------------------------- |
| `req_id`       | u64?     | Echoed if present in request                                |
| `status`       | string   | `"success"` or `"error"`                                    |
| `duration_ms`  | u128     | Wall time inside the orchestrator                           |
| `id`           | u32?     | Terminal id, when applicable                                |
| `content`      | string?  | Screen contents, list output                                |
| `error`        | string?  | Detailed message, present iff `status == "error"`           |
| `sub_results`  | array?   | Present only on `Batch` responses                           |

## Commands

### `Spawn`

```json
{"type": "Spawn", "payload": {
  "title": "build",
  "command": "cargo build",
  "timeout_ms": 600000,
  "max_lines": 5000,
  "visible": false
}}
```

All payload fields except `title` are optional. `visible` precedes any batch-level
visibility, which itself precedes the orchestrator-level default.

Response: `{"id": 42, ...}`.

### `ScreenWrite`

```json
{"type": "ScreenWrite", "payload": {"id": 1, "text": "vim<Enter>"}}
```

Recognised shortcuts in `text`:

| Token   | Bytes        | Token     | Bytes      |
| ------- | ------------ | --------- | ---------- |
| `<C-c>` | `0x03`       | `<Up>`    | `\x1b[A`   |
| `<C-d>` | `0x04`       | `<Down>`  | `\x1b[B`   |
| `<Tab>` | `0x09`       | `<Right>` | `\x1b[C`   |
| `<Enter>` | `\r\n`     | `<Left>`  | `\x1b[D`   |
| `<Esc>` | `\x1b`       | `<C-a>`   | `0x01`     |
| `<C-e>` | `0x05`       |           |            |

### `ScreenRead`

```json
{"type": "ScreenRead", "payload": {"id": 1}}
```

Returns the full visible screen contents in `content`.

### `ScreenControl`

```json
{"type": "ScreenControl", "payload": {"id": 1, "action": "minimize"}}
```

Actions: `minimize`, `maximize`, `restore`, `close`, `pin`, `unpin`, `menu`.

### `ScreenClose`

```json
{"type": "ScreenClose", "payload": {"target": "all"}}
{"type": "ScreenClose", "payload": {"id": 1, "target": "single"}}
```

`target = "all"` closes every terminal owned by the connection.

### `List`

```json
{"type": "List", "payload": {}}
```

Returns terminal IDs owned by the connection in `content` as a JSON array string.

### `Wait`

```json
{"type": "Wait", "payload": {"ms": 250}}
```

### `WaitUntil`

```json
{"type": "WaitUntil", "payload": {"id": 1, "pattern": "Ping statistics", "timeout_ms": 10000}}
```

`pattern` is a literal substring match against the rendered screen. Returns `error`
on timeout.

### `Batch`

```json
{"type": "Batch", "payload": {
  "stop_on_error": true,
  "visible": false,
  "commands": [ ... ]
}}
```

Returns **one** response with `sub_results: [CommandResult, ...]`. Aggregate `status` is
`"error"` if any sub failed, regardless of `stop_on_error`.

## Error envelope

```json
{
  "req_id": 9,
  "status": "error",
  "duration_ms": 0,
  "error": "parse: unknown variant `Spwan`, expected one of `Spawn`, `ScreenWrite`, ..."
}
```

Causes that produce errors (never silent drops):

| Cause                       | `error` prefix    |
| --------------------------- | ----------------- |
| JSON parse failure          | `"parse: ..."`    |
| Unknown terminal `id`       | `"terminal: ..."` |
| `WaitUntil` timeout         | `"timeout: ..."`  |
| OS / PTY error              | `"io: ..."`       |
| Window control failure      | `"window: ..."`   |
| Internal panic (recovered)  | `"panic: ..."`    |

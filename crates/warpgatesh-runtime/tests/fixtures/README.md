# Warpgate user API fixtures

These are synthetic responses, not captured production data. Hostnames use
`example.test`, usernames and UUIDs are invented, and no fixture contains a
credential. Test requests use a synthetic token.

Each `info.json` contains the required fields of the upstream `Info` schema
plus the authenticated metadata consumed by WarpgateSH. Each `targets.json`
includes all seven `TargetKind` values, required `TargetSnapshot` fields and
one synthetic target group. Optional fields not needed by these tests are
omitted. The 0.29.1 info fixture deliberately omits the removed
`minimize_password_login` field and includes the new MFA fields.

Source: `warpgate-web/src/gateway/lib/openapi-schema.json` in the official
[Warpgate repository](https://github.com/warp-tech/warpgate), reviewed 2026-10-02.

| Version | Immutable source commit |
| --- | --- |
| 0.27.5 | [a28faaa4f99e6a2a7bbb4db359b18b539536f78b](https://github.com/warp-tech/warpgate/blob/a28faaa4f99e6a2a7bbb4db359b18b539536f78b/warpgate-web/src/gateway/lib/openapi-schema.json) |
| 0.28.6 | [525c7caf2219d5f5e3913b5732e4cbad5d15cd34](https://github.com/warp-tech/warpgate/blob/525c7caf2219d5f5e3913b5732e4cbad5d15cd34/warpgate-web/src/gateway/lib/openapi-schema.json) |
| 0.29.1 | [54f93c807be2c161a94c0df764242849125161a8](https://github.com/warp-tech/warpgate/blob/54f93c807be2c161a94c0df764242849125161a8/warpgate-web/src/gateway/lib/openapi-schema.json) |

When adding a version, check the actual HTTP route mounting and token security
scheme too. Verify fixture required fields, types, enum values and target UUIDs
against its schema; do not infer future API shapes or label synthetic fixtures
as live-server validation. See the [compatibility review](../../../../docs/warpgate-api-compatibility.md).

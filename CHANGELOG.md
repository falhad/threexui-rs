# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project loosely tracks the upstream [3x-ui panel](https://github.com/MHSanaei/3x-ui) version.

## [2.9.4] – 2026-05-03

Live-tested against 3x-ui **v2.9.2** and **v2.9.3** panels. All scenarios green
on both versions.

### Added

- **Outbound proxy support.** `ClientConfig::builder()` now exposes
  `.proxy(url)`, `.proxy_auth(user, pass)`, and `.no_proxy()`. Accepts
  `http://`, `https://`, `socks5://`, and `socks5h://` URLs. Bad URLs are
  rejected at `build()` time as `Error::Config`. Live-verified through
  `tinyproxy` (HTTP) and `go-socks5-proxy` (SOCKS5h).
- **`Error::EndpointNotFound(String)`** variant — surfaces HTTP 404 from a
  panel that's too old (e.g. `inbounds.copy_clients` on v2.9.2) with a clear
  message instead of an opaque JSON-decode error.
- **Centralized response decoder** (`client::read_api_response`) — every API
  call now routes through one parser that:
  - Maps HTTP 404 → `Error::EndpointNotFound`.
  - Maps empty body → `Error::Api("empty response body (HTTP <code>)")`.
  - Maps non-JSON error pages (e.g. nginx HTML 5xx) → `Error::Api("HTTP <code> — non-JSON body: …")`.
- **Live integration testers**:
  - `examples/live_test.rs` — full smoke test (51 assertions) of every public method.
  - `examples/scenarios.rs` — 16 production scenarios / 39 assertions covering
    auth flows, multi-protocol create (vless/vmess/trojan/shadowsocks),
    `find_client_by_uuid`, bulk client creation, renew, disable/enable,
    reset-uuid, data-limit increase, traffic resets, concurrent reads,
    special-character round-trips, cross-inbound search, and negative paths.
  - `examples/proxy_test.rs` — verifies the proxy wiring end-to-end.
- **Serde robustness suite** at `tests/serde_robustness.rs` — 23 tests pinning
  decoder behaviour against real-world 3x-ui payloads.

### Fixed

- **`InboundClient.total_gb` decode failure.** The field was renamed by
  `serde(rename_all = "camelCase")` to `totalGb`, but the panel sends
  `totalGB`. Decoding any inbound with clients fell back to "missing field
  `totalGb`". Added explicit `#[serde(rename = "totalGB")]`. Affected every
  downstream call that parsed `settings.clients` after `.get()`.
- **`InboundClient.tg_id` decode failure.** Panel sends `tgId` as either an
  integer, a numeric string (`"77313385"`), `null`, or omits it entirely. The
  field was a plain `i64`. Added a flexible deserializer that accepts all
  shapes; serializes back as `i64`.
- **`Inbound.client_stats: null` rejected.** `/panel/api/inbounds/get/{id}`
  returns `clientStats: null`, which serde refused for `Vec<ClientTraffic>`.
  This was the root cause of the production `find_client_by_uuid` and
  `find_client` bugs reported downstream — every renew / disable / data-limit
  / reset-uuid / delete operation routed through `.get()` and failed. Now
  treats `null` as `[]`.
- **`xray.get_setting` decode failure.** The panel returns `obj` as a
  JSON-encoded *string* of the inner settings, not as a nested object. Now
  parsed in two steps.
- **`custom_geo.aliases` decode failure.** Returned `Vec<String>`, but the
  panel sends `{ "geosite": null, "geoip": null }`. New `CustomGeoAliases`
  type with `null`-tolerant fields; method now returns it.
- **`AllSetting.ldap_default_total_gb` decode failure.** Same `totalGB`
  rename issue as `InboundClient`. Now `#[serde(rename = "ldapDefaultTotalGB")]`.
- **`server.logs` / `server.xray_logs` "empty response" error.** When no logs
  are available, the panel returns `obj: null`. Both methods now treat that
  as an empty `Vec<String>` instead of erroring.

### Changed

- **`reqwest` upgraded 0.12 → 0.13.** Feature names adjusted: `rustls-tls`
  renamed to `rustls`, and the `form` and `socks` features are now
  explicitly enabled. No public API change.
- All other dependencies bumped to their latest semver-compatible versions.

### Compatibility

| Panel version | Status |
| --- | --- |
| 3x-ui v2.9.2 | ✅ All scenarios pass. `inbounds.copy_clients` returns `Error::EndpointNotFound` (endpoint not present in v2.9.2). |
| 3x-ui v2.9.3 | ✅ All scenarios pass. |

### Test counts

- **75** in-process tests (51 unit + 23 serde robustness + 1 doc-test).
- **51** assertions per panel in `live_test`, **39** in `scenarios`.
- All green on v2.9.2 and v2.9.3 panels.

---

## [2.9.3] – earlier

Initial public release targeting 3x-ui v2.9.3.

---
phase:
  current: 6
  total: 6
  names:
    1: "Research & planning"
    2: "Frontier tab implementation"
    3: "Routines tab implementation"
    4: "Signals tab implementation"
    5: "Actions project filter"
    6: "Review and polish"
  status:
    1: done
    2: done
    3: done
    4: done
    5: done
    6: done
---

# ADR-0035: Custom KodeHold Viewer

## Status

Deprecated

**Version:** 2.1
**Last Updated:** 2026-06-06
**Phase:** Phase 6 (Observability) — builds on Phases 3–5 (Actions, Routines, Crystals + Signals) to add a dedicated interactive viewer with Frontier, Routines, Signals, and Memory Types views.

### Changelog

| Version | Date | Changes |
|---------|------|---------|
| 2.1 | 2026-06-06 | Added Memory Types tab — fetches all semantic facts and procedural memories with search/filter. Dashboard cards link to full view when >5 items exist. |
| 2.0 | 2026-06-02 | Addressed review blockers: signals agentId REQUIRED, frontier blockers clarified, input sanitization + CORS + error handling section added, Google Fonts removed, routines schema corrected, API query parameters documented |
| 1.0 | — | Initial proposal |

## Context

### The Problem

Agentmemory ships a built-in web viewer at port 3113 (`/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/viewer/index.html`). This viewer is a self-contained single HTML file with 12 tabs: Dashboard, Graph, Memories, Timeline, Sessions, Lessons, Actions, Crystals, Audit, Activity, Profile, and Replay. It is a generic database browser — it shows raw data from agentmemory's storage but does **not** surface KodeHold-specific operational views.

KodeHold now operates a sophisticated action-based delegation system (ADR-0031), routine templates (ADR-0032), inter-agent signals (ADR-0033), and auto-crystallization. The built-in viewer is missing four critical views:

1. **Frontier** — The `GET /agentmemory/frontier` endpoint returns 7 unblocked actions sorted by priority/score, with blocker information and lease status. The built-in Actions tab shows *all* 50 actions with no priority sorting and no score column, making it impossible to see "what should I work on next?"

2. **Routines** — The `GET /agentmemory/routines` endpoint returns 4 registered routine templates (kodehold-adr-flow, kodehold-implement-flow, kodehold-bugfix-flow, kodehold-ship-gate) with their step DAGs, dependencies, and frozen status. The built-in viewer has no routines tab at all.

3. **Signals** — The `GET /agentmemory/signals?agentId=X` endpoint returns threaded inter-agent messages with types (info, request, response, alert, handoff), replyTo threading, and read status. The built-in viewer has no signals tab, making it impossible to trace agent-to-agent communication chains.

4. **Actions project filter** — The Actions tab shows all actions across all projects. With 50+ actions spanning kodehold, flow-test, and adopted workspaces, there is no way to filter by project to reduce noise.

### Prior Art

| Approach | Description | Status |
|----------|-------------|--------|
| ADR-0034 (Workflow Monitor) | Python script generating static HTML at `docs/dashboard/workflow.html` | Accepted — server-generated snapshot, not interactive |
| Built-in viewer (port 3113) | Generic database browser with 12 tabs | Exists — no KodeHold-specific views |

ADR-0034's workflow monitor is a **snapshot** tool — it runs as a Python script and generates a static HTML page that you refresh. This ADR proposes an **interactive** viewer — a dynamic single-page application that fetches live data from the REST API and lets you filter, sort, and drill down in real time. They are complementary.

### Why Not Patch the Built-in Viewer?

The viewer is embedded in agentmemory's `dist/index.mjs` and `dist/viewer/index.html`. Modifying these files would:
- Break on every `npm update` or reinstall of agentmemory
- Require maintaining a fork of agentmemory's dist
- Risk introducing regressions in the stable built-in viewer
- Mix concerns (generic database browser vs KodeHold-specific operations)

### Why Not Use the REST API in AM's Existing Viewer?

The built-in viewer communicates with the server via WebSocket, not the REST API, and its tab system is not extensible. Adding new tabs would require modifying the compiled `index.mjs` — the same patching problem.

### Key Forces

1. **Non-invasive** — Must never modify agentmemory's files
2. **Self-contained** — One file, zero dependencies, easy to deploy
3. **Interactive** — Live data, filtering, sorting — not a static snapshot
4. **Stylistically consistent** — Should look like it belongs alongside the built-in viewer
5. **Graceful degradation** — If agentmemory daemon is down, show helpful error messages, not a blank page
6. **Zero token cost** — Runs in the browser, no LLM inference required
7. **Easy to serve** — Works both standalone (file:/// or python3 -m http.server) and proxied behind the built-in viewer

## Decision

Build a custom KodeHold viewer as a standalone single HTML file with embedded CSS and JS, served from `tools/viewer/index.html`.

### Architecture: Option A — Standalone HTML file (Chosen)

| Criteria | Option A: Standalone HTML | Option B: Proxy via Node.js | Option C: Patch built-in |
|----------|--------------------------|----------------------------|--------------------------|
| **Risk of breaking AM** | None | Low | High |
| **Deployment complexity** | None (open file) | Medium (npm install, run proxy) | None (already deployed) |
| **Dependencies** | Zero | Node.js + express + http-proxy | None |
| **CORS handling** | Needed (port 3111 → viewer) | Handled by proxy | Not needed (same origin) |
| **Maintenance** | Low | Medium | High (each AM update) |
| **Auth/session sharing** | None (separate origin) | Possible (same origin) | Automatic |
| **Upgrade safety** | Perfect | Good | Breaks on update |

**Decision Rationale:** Option A is chosen because the zero-maintenance, zero-dependency, zero-risk profile outweighs the CORS inconvenience. CORS is a solvable problem — either the viewer runs behind a simple proxy (a 3-line nginx config or the built-in viewer's static file server) or the browser is launched with `--disable-web-security` for local development.

### Viewer Tab Design

The custom viewer adds 4 new tabs and enhances 2 existing tabs, relative to the built-in viewer:

| Tab | Source | Description |
|-----|--------|-------------|
| **Dashboard** | Built-in viewer **+ enhanced** | Stats overview — Semantic Memory and Procedural Memory cards now show "Show all N →" links when >5 items exist, linking to the Memory Types tab |
| **Graph** | Built-in viewer | Unchanged — knowledge graph visualization |
| **Memories** | Built-in viewer | Unchanged — memory entries browser |
| **Memory Types** | **New** | All semantic facts and procedural memories with search/filter |
| **Timeline** | Built-in viewer | Unchanged — chronological observation view |
| **Sessions** | Built-in viewer | Unchanged — session list |
| **Lessons** | Built-in viewer | Unchanged — lessons table |
| **Actions** | Built-in viewer **+ project filter** | Enhanced with project dropdown filter |
| **Frontier** | **New** | Unblocked actions sorted by score |
| **Routines** | **New** | Routine templates with step DAGs |
| **Signals** | **New** | Threaded inter-agent messages |
| **Crystals** | Built-in viewer | Unchanged — crystal digests |
| **Audit** | Built-in viewer | Unchanged — audit trail |
| **Activity** | Built-in viewer | Unchanged — activity log |
| **Profile** | Built-in viewer | Unchanged — user/project profile |
| **Replay** | Built-in viewer | Unchanged — session replay |

### Frontier Tab Design

- **Data source:** `GET /agentmemory/frontier`
- **Important:** The frontier API returns ONLY unblocked actions (blocked actions are filtered out by the server). This tab intentionally shows only the actions that are ready to work on.
- **Table columns:** Score, Title, Status, Priority, Project, Leased, Created
- **Sort default:** By score descending
- **Filter:** Project dropdown (populated from unique project values in the data)
- **Row detail:** Click to expand full action object (description, ID, tags, metadata)
- **Visual indicators:**
  - Leased actions show a green dot + "leased" badge
  - Score shown as a numeric badge with color gradient (green for 90+, yellow for 70–89, gray below)
- **Note on blocked actions:** To view blocked actions, switch to the Actions tab and filter by project — all actions (including blocked) are visible there. Blocked actions can be identified by inspecting the `requires` field (unresolved dependencies).

### Routines Tab Design

- **Data source:** `GET /agentmemory/routines`
- **Card layout:** Each routine is a collapsible card showing:
  - Name + ID (monospace)
  - Tags (from routine metadata)
  - Updated timestamp
  - Frozen status badge
  - Step count
- **Expanded view:** Step table with Order, Title, Team (from tags), Priority, Dependencies, Description
- **Dependency visualization:** For each step, show "depends on step N" as text links
- **Run history:** Query actions with `routineId` matching this routine's ID to show instantiation history

### Signals Tab Design

- **Data source:** `GET /agentmemory/signals?agentId=<agent>` — agentId is REQUIRED by the API (returns 400 if missing). The viewer iterates over known agents:
  ```javascript
  const AGENTS = ['director', 'architects', 'engineers', 'reviewers', 'testers', 'fls', 'scribes'];
  ```
- **Initial view:** Agent selector dropdown populated from `AGENTS` array
- **Threaded view:** Signals grouped by `threadId`, sorted by `createdAt`
- **Each signal shows:**
  - From → To with direction arrow
  - Type badge (color-coded: info=blue, request=yellow, response=green, alert=red, handoff=purple)
  - Content text
  - Timestamp
  - Read/unread status
  - Reply indicator (if `replyTo` is set, link to parent signal)
- **Filter by:** Agent (from/to), type, unread only

### Memory Types Tab Design

- **Data source:** `GET /agentmemory/semantic` and `GET /agentmemory/procedural`
- **Two sub-sections:**
  - **Semantic Facts** — table of all semantic memories extracted by the consolidation pipeline, with columns: Content, Type, Concepts, Confidence, Created
  - **Procedural Memories** — table of all procedural workflow routines extracted by the consolidation pipeline, with columns: Name, Trigger, Steps, Confidence, Created
- **Search/filter:** Each sub-section has a search input that filters rows by text match
- **Dashboard integration:** The Dashboard tab's Semantic Memory and Procedural Memory cards show "Show all N →" links when there are more than 5 items, linking directly to the Memory Types tab
- **Empty state:** Shows "No semantic facts found" or "No procedural memories found" when the API returns empty results

### Actions Tab Enhancement

- **Add project filter dropdown** above the table
- **Populate dropdown** from unique `project` values across all actions
- **Default:** Show all (no filter)
- **On change:** Filter table rows by project match
- **Preserve** all existing table columns and functionality from the built-in viewer

### REST API Endpoints Consumed

| Endpoint | Method | Query Parameters | Response Shape | Used By |
|----------|--------|-----------------|---------------|---------|
| `/agentmemory/actions` | GET | status, project, tags, limit | `{ actions: [...] }` | Actions tab, Frontier tab (fallback), Routines tab (run history) |
| `/agentmemory/frontier` | GET | limit, project, agentId | `{ frontier: [{action, score, blockers, leased}] }` | Frontier tab |
| `/agentmemory/routines` | GET | frozen | `{ routines: [{id, name, steps, frozen, ...}] }` | Routines tab |
| `/agentmemory/signals?agentId=X` | GET | agentId (REQUIRED), unreadOnly, threadId, limit | `{ signals: [{id, from, to, type, content, threadId, ...}] }` | Signals tab |
| `/agentmemory/semantic` | GET | (none) | `{ semantic: [{content, type, concepts, confidence, createdAt, ...}] }` | Memory Types tab |
| `/agentmemory/procedural` | GET | (none) | `{ procedural: [{name, trigger, steps, confidence, createdAt, ...}] }` | Memory Types tab |
| `/agentmemory/crystals` | GET | (none) | `{ crystals: [...] }` | (optional — matches built-in) |

The viewer does **not** call any write endpoints. It is read-only.

### Styling

Match the built-in viewer's design system exactly:
- **CSS variables** (`--bg`, `--ink`, `--accent`, `--font-*`, etc.) — either import from the built-in viewer or replicate the subset needed
- **Typography:** System font stack only — no external font loads (avoids CSP issues). Use `serif` for display/body, `system-ui, -apple-system, sans-serif` for UI, and `'Courier New', Consolas, monospace` for code. The built-in viewer removed Google Fonts for CSP compliance; this viewer follows the same constraint.
- **Color palette:** Cream background (#F9F9F7), black borders, red accent (#CC0000), muted inks
- **Tab bar:** Same uppercase, small-caps, letter-spaced style with red active indicator
- **Tables:** Same clean bordered style with alternating row backgrounds
- **Badges:** Same uppercase monospace badges for status/type/score
- **Dark mode:** Respect `prefers-color-scheme` and `data-theme` attribute — use the same dark theme variables

### CORS Strategy

Since the viewer runs on a different port than agentmemory's REST API (port 3111), CORS headers are required. Three options:

| Option | Mechanism | Pros | Cons |
|--------|-----------|------|------|
| **C1. Agentmemory CORS config** | Set `AGENTMEMORY_CORS=true` or add CORS middleware | Cleanest, no extra infra | Requires agentmemory config change (one-time) |
| **C2. Reverse proxy** | nginx or Caddy proxies `/agentmemory/*` → port 3111 on same origin as viewer | No CORS needed | Additional dependency |
| **C3. Serve viewer from AM's static dir** | Place viewer in AM's viewer/ directory alongside index.html | Same origin, no CORS | Slightly invasive (but just a file copy) |

**Recommendation:** C1 (agentmemory CORS) for development, C2 (reverse proxy via the built-in viewer server) for production. Agentmemory already has a built-in HTTP server; if it can be configured to serve static files from a custom directory, the viewer can be placed there for same-origin access.

If the viewer is opened as `file:///tools/viewer/index.html`, `fetch()` to `http://localhost:3111` will be blocked by CORS. The viewer should detect this and display a helpful message: *"CORS error: Open this page via http:// or configure agentmemory CORS."*

### Security and Error Handling

#### Input Sanitization

All user-facing input fields (search boxes, filter dropdowns, project text input) must sanitize values before rendering or using in API calls:

- **Search/filter fields:** Strip HTML tags from input before display. Use `textContent` instead of `innerHTML` for rendering user-controlled data.
- **API query parameters:** URL-encode all user-supplied values before appending to fetch URLs — especially project names and search terms that may contain spaces or special characters.
- **Content rendering:** When displaying API response data (action descriptions, signal content, routine names), always use `textContent` to prevent XSS if stored data contains HTML.

#### Authentication and Authorization

The custom viewer inherits the same security model as the built-in agentmemory viewer:

- **Localhost-only access:** Both viewers are designed exclusively for local development and debugging. They bind to localhost and are not exposed to network access.
- **No authentication layer:** Neither viewer implements authentication. Access control is provided by the localhost network boundary.
- **Same origin policy:** The viewer fetches from `http://localhost:3111/agentmemory/*` — this is localhost-to-localhost and should never be proxied through a public-facing endpoint.

#### Error Handling

Each tab's data fetch must handle four failure modes gracefully:

| Scenario | Behavior |
|----------|----------|
| API unreachable (daemon down) | Show "API unreachable — is agentmemory running?" with a retry button |
| HTTP error (4xx/5xx) | Show the status code and error body from the response |
| CORS error (cross-origin block) | Display the CORS help message (see above) with setup instructions |
| Empty response (no data) | Show "No data" with a hint about what to check (e.g., "No routines registered" or "No signals found for this agent") |

Each tab fetches independently — a failure in one tab does not affect others. A shared `fetchJSON` utility function wraps all fetch calls with this error handling.

### File Layout

```
tools/viewer/
  index.html          — Single HTML file with embedded CSS and JS (the viewer)
  README.md           — Quick-start instructions
```

The viewer is a single HTML file for maximum portability — matching the built-in viewer's deployment model.

### Build and Dependency Strategy

- **Zero build step** — the viewer is handwritten HTML/CSS/JS (ES module-free, works in all modern browsers)
- **No npm packages** — no webpack, no bundler, no node_modules
- **No framework** — vanilla JS, no React/Vue/Svelte (keeps the file size small and load time instant)
- **Polyfill-free** — targets modern browsers (Chrome, Firefox, Safari, Edge — last 2 versions)

## Consequences

### Positive

1. **Four missing views, zero risk.** Frontier, Routines, Signals, and Memory Types become visible without touching agentmemory's files.
2. **Project filter for Actions.** Reduces noise when monitoring specific workspaces.
3. **Self-contained.** One HTML file — `scp` it anywhere, open it anywhere.
4. **Complementary to ADR-0034.** The workflow monitor (Python, snapshot) and this viewer (JS, interactive) serve different use cases — one for Slack/email reports, one for interactive debugging.
5. **Instant deployment.** No build, no install, no restart — just open the file.
6. **Graceful degradation.** Each view fetches independently; if an endpoint is down, that view shows "Data unavailable" and the rest still works.
7. **Stylistically consistent.** Matches the built-in viewer's design — users see one visual language across both viewers.
8. **Zero token cost.** Runs entirely in the browser at zero LLM inference cost.

### Negative

1. **CORS workaround required.** The viewer fetches from port 3111, which is a different origin. A one-time CORS config or reverse proxy setup is needed.
2. **Not integrated into AM's built-in navigation.** Users must remember a second URL or bookmark `tools/viewer/index.html`.
3. **Manual refresh.** No WebSocket push — data is current only at fetch time. Mitigated by an auto-refresh toggle (e.g., refresh every 10 seconds).
4. **No auth.** The viewer has no authentication — it relies on agentmemory being on localhost.
5. **Schema coupling.** If agentmemory's REST API response shape changes, the viewer's JSON parsing must be updated.

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| API schema changes | Medium | Medium | Version-and-release notes; test viewer against each AM upgrade |
| CORS setup friction | Medium | Low | Document 3 options clearly in README; include a `--disable-web-security` Chromium launch command |
| Viewer becomes stale (no updates) | Medium | Low | It's a static file — no security surface; works as-is until API changes |
| Duplication with ADR-0034 | Low | Low | ADR-0034 is snapshot; this is interactive — documented distinction in both ADRs |

### Follow-up

- [x] Create `tools/viewer/README.md` with quick-start instructions and CORS setup guide
- [x] Implement Frontier tab (table + project filter + score badges)
- [x] Implement Routines tab (card layout + expandable step DAGs)
- [x] Implement Signals tab (agent selector + threaded view + type badges)
- [x] Add project filter dropdown to Actions tab
- [x] Add Memory Types tab (semantic facts + procedural memories with search/filter)
- [x] Add "Show all N →" links from Dashboard to Memory Types tab
- [ ] Add auto-refresh toggle (5s/10s/30s/off)
- [ ] Test with agentmemory daemon running and stopped (degradation)
- [ ] Register viewer path in opencode.json as optional Director tool reference

## ADR References

- **ADR-0031** (Actions + Crystals) — action/frontier system that the Frontier tab surfaces
- **ADR-0032** (Routine Templates) — routine templates that the Routines tab surfaces
- **ADR-0033** (Crystals + Signals) — signal system that the Signals tab surfaces
- **ADR-0034** (Workflow Monitor) — complementary snapshot approach; this ADR's interactive viewer fills the same observability gap from a different angle
- **ADR-0014** (Status Dashboard) — foundational HTML generation pattern
- Built-in viewer: `/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/viewer/index.html`
- REST API base: `http://localhost:3111/agentmemory/`
- Built-in viewer base: `http://localhost:3113/`

# ADR 0002: Nonce-Isolated, SVG-Safe Markdown Rendering

- **Status**: Accepted
- **Scope**: `dtnpm-mod-chat` (`ui/src/utils/markdown.ts`, `ui/src/ChatView.tsx`)

## Context
Chat messages rendered with `renderMarkdown` relied on predictable static placeholders (`@@HDYTOKEN...@@`), a complex backtracking link regex, autolink conversion running after HTML escaping, and allowed `data:image/svg+xml` as well as unconditional remote HTTP(S) image fetches (enabling active-content risks and privacy leaks).

## Decision
Align `ui/src/utils/markdown.ts` with `dtnpm-web-core`:
1. Strip `\x00` from untrusted input and generate a per-invocation random alphanumeric nonce (`\x00DTNMD${nonce}X${counter}\x00`) for placeholder isolation.
2. Process `<https://...>` / `<dtn://...>` angle-bracketed autolinks and bare autolinks prior to `escapeHtml`, and replace nested quantifiers in `linkRegex` with a linear pattern.
3. Restrict `isSafeImageUrl` to raster `data:image/(png|jpeg|jpg|gif|webp);base64,...` URIs and safe relative paths by default, requiring explicit `allowExternalImages: true` opt-in for remote `http(s)://` images (`ChatView.tsx` keeps both image flags off by default).

## Consequences
- Forged placeholder injection, SVG script payloads, ReDoS on pathological link syntax, and unsolicited remote image beacons are prevented while preserving explicit opt-in flexibility for trusted contexts.

---
name: changelog
description: Create a changie changelog entry for a code change. Use when asked to add a changelog entry, record a change, or after completing a feature/fix/refactor.
argument-hint: "[description of the change]"
---

Create a changelog entry by writing a YAML file to `.changes/unreleased/`.

## Changie configuration

This project uses [changie](https://changie.dev/) for changelog management. The configuration is in `.changie.yaml`.

### Available kinds

Choose the most appropriate kind for the change:

| Kind | Auto bump | Use for |
|------|-----------|---------|
| `Added` | minor | New features, new API surface |
| `Fixed` | patch | Bug fixes |
| `Performance` | patch | Performance improvements |
| `Changed` | patch | Changes to existing behavior, MSRV bumps |
| `Reverted` | patch | Reverted changes |
| `Dependencies` | patch | Dependency updates |
| `Security` | patch | Security fixes |

### File format

Each entry is a YAML file in `.changes/unreleased/` with this structure:

```yaml
kind: Added
body: |-
    Short summary of the change

    Optional longer description with more detail. This can be multiple lines.
    Lines in the same paragraph are joined with spaces in the rendered changelog.
time: 2026-03-14T00:00:00.000000-08:00
```

### File naming

Use the pattern: `{Kind}-{slug}.yaml`

- The slug should be a short kebab-case description of the change
- Examples: `Added-message-support.yaml`, `Fixed-cache-race-condition.yaml`, `Dependencies-bump-semver.yaml`

### Body format

The `body` field uses a block scalar (`|-`). The first line is the summary (rendered as an `####` heading in the changelog). Subsequent lines form an optional description paragraph.

- First line: concise summary of the change (acts as heading)
- Remaining lines: optional detailed explanation
- Keep it useful for consumers of the crate — focus on what changed and why it matters

### Time field

Use the current date with a timestamp. Detect the user's local timezone offset (e.g., via the `date` command) and use that in the time field.

## Instructions

1. Determine the appropriate `kind` based on the nature of the change
2. Write a clear, user-facing summary as the first line of `body`
3. Optionally add a longer description on subsequent lines
4. Create the YAML file in `.changes/unreleased/` with the naming convention above
5. If `$ARGUMENTS` is provided, use it to inform the changelog entry content

$ARGUMENTS

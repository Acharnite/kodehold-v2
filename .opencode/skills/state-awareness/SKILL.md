---
name: state-awareness
description: Lifecycle state check preamble and mismatch reporting protocol. Load this skill before any task to verify the project is in the correct state for the work.
---

# State Awareness Protocol

Use this protocol BEFORE starting any work. It ensures you only act in the correct lifecycle state and report mismatches properly.

## Step 1: Check Current State

Run one of these to determine the current lifecycle state:

```
# For KodeHold itself or workspace project root:
python3 scripts/gate.py --status

# For workspace projects specifically:
python3 scripts/workspace.py state <name>
```

Read the `STATE=` line from the output.

## Step 2: Verify Allowed Phase

Each team has specific phases where their work is valid (defined per-agent below). If the current state matches one of your allowed phases → proceed with the task.

## Step 3: Refuse on Mismatch

If the project is in the wrong state for the requested work:

1. **DO NOT** attempt the work
2. **Report** to the Director with a clear message:
   - Current state
   - What state is required
   - What gate or action is needed to get there
   - A concrete example message
3. **Wait** for the Director to handle the transition

## Step 4: Workspace Awareness

For workspace projects (`workspaces/<name>/`):
- Run `python3 scripts/workspace.py state <name>` to check workspace state
- The workspace `.kodehold-state` may differ from the root project state
- Adopted projects have `ADOPTED=true` — this may relax some phase restrictions

## Agent-Specific Allowed States

> **Each agent defines its own allowed states below this skill reference in its `.md` file.**
> The allowed states are NOT in this shared skill — they are per-agent configuration.

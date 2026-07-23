import { tool } from "@opencode-ai/plugin"
import { $ } from "bun"

export default tool({
  description: "Load context from memory and graphify before responding. Run this at the start of EVERY turn to avoid asking questions that are already answered.",
  args: {
    query: tool.schema.string().describe("User's question or topic to search for"),
  },
  async execute(args, context) {
    const { query } = args
    const results: string[] = []

    // Step 1: graphify query
    try {
      const graphifyResult = await $`graphify query "${query}" --budget 1500`.text()
      if (graphifyResult.trim()) {
        results.push(`## Graphify Results\n${graphifyResult.trim().slice(0, 1200)}`)
      }
    } catch (e) {
      results.push("## Graphify Results\nNo graph data available or graphify query failed.")
    }

    // Step 2: memory search
    try {
      const memoryResult = await $`opencode-mem search "${query}" --scope project --limit 5`.text()
      if (memoryResult.trim()) {
        results.push(`## Memory Results\n${memoryResult.trim().slice(0, 1200)}`)
      }
    } catch (e) {
      results.push("## Memory Results\nNo memory data available or search failed.")
    }

    // Step 3: Check STATE.md for current context
    try {
      const stateResult = await $`cat ${context.worktree}/STATE.md 2>/dev/null || echo "STATE.md not found"`.text()
      if (stateResult.trim() && !stateResult.includes("not found")) {
        results.push(`## Current State (STATE.md)\n${stateResult.trim().slice(0, 600)}`)
      }
    } catch (e) {
      // STATE.md not found - skip
    }

    if (results.length === 0) {
      return "No context found for: " + query
    }

    return results.join("\n\n---\n\n")
  },
})

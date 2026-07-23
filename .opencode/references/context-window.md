# Context Window Pressure Protocol

Before each Task tool delegation, the Director MUST estimate current context size:

1. **Estimate current context** — count approximate tokens used in the current session:
   - Each prior message in the conversation: ~500 tokens average
   - Current task prompt: estimate based on length
   - Loaded files/context: approximate from file sizes
   - Result: rough estimate of current context usage

2. **Compare against model limit** — typical limits:
   - Large context (Claude, GPT-4): 100K tokens
   - Small context (Ollama 32K): 32K tokens

3. **Act based on pressure level:**
   - If estimated usage < 60% of limit → proceed normally
   - If 60-80% → warn user: "Context at ~<X>%. Consider compression soon."
   - If 80-90% → suggest compression: "Context at ~<X>%. Recommend session compression before next delegation."
   - If > 90% → force compression via Scribes before proceeding. Delegate to Scribes to create a session summary, then suggest starting fresh session with /resume.

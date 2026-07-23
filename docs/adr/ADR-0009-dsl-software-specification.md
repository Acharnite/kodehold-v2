---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0009: DSL for Software Specification and Compilation

## Status

Proposed

## Context

In KodeHold v1.0, AI agents implement features by interpreting natural-language
task descriptions:

```
Director: "Engineers, implement a RegimeClassifier that takes OHLCV data
          and returns a market regime classification."

Engineers: [Generates code based on context + prompt]
```

This works but has fundamental limitations:

1. **Ambiguity**: "Classify market regime" is underspecified. What regimes?
   What algorithm? What time complexity? The agent fills gaps with guesses.
2. **Non-determinism**: Same prompt, different day, different implementation.
3. **No guarantees**: Nothing enforces that the generated code matches the
   intent beyond manual review.
4. **Token cost**: Every implementation round-trips through an LLM, even for
   well-understood problems with known solutions.

The insight: most software is "fill in the blanks" — types, interfaces, data
flow, and constraints are specified. The implementation connects the dots.
If we can formalize the specification, we can compile it.

## Decision

Introduce `.khold` — a domain-specific language for software specification.
The compiler reads a `.khold` file and generates the implementation.

### Design Philosophy

The DSL specifies WHAT, not HOW:

```
WHAT:  A function that classifies market data into regimes
TYPES:  Input = Vec<OHLCV>, Output = Regime (enum {Trending, Ranging, Volatile})
CONSTRAINTS: O(n) time, O(1) space, pure (no side effects)
TESTS:  trending_up → Trending, flat → Ranging, high_atr → Volatile
```

The compiler decides HOW based on the constraint solver (ADR-0006) and pattern
database. If no pattern matches, it falls back to AI generation with the full
typed specification as context — much more constrained than a natural-language
prompt.

### Language Structure

```
// Project declaration
project "my-trading-bot" {
    version = "0.1.0"
    language = "rust"
    edition = "2024"
}

// Type definitions
type Price = f64
    constraint: >= 0.0

type OHLCV = struct {
    open: Price,
    high: Price,
    low: Price,
    close: Price,
    volume: u64,
}

// Enum with exhaustive variants
enum Regime {
    Trending { direction: TrendDirection },
    Ranging { upper: Price, lower: Price },
    Volatile { atr: f64 },
}

enum TrendDirection { Up, Down }

// Interface (trait in Rust, interface in Go, ABC in Python)
interface MarketDataFeed {
    fn fetch(symbol: String, from: Timestamp, to: Timestamp)
        -> Result<Vec<OHLCV>, FeedError>
}

// Function specification with constraints
fn classify_regime(data: Vec<OHLCV>) -> Regime {
    constraint {
        time: O(n)
        space: O(1)
        pure: true
    }

    test "trending_up": {
        let data = mock_uptrend(20, Price(100.0), Price(110.0))
        assert classify_regime(data) is Regime::Trending { direction: Up }
    }

    test "ranging": {
        let data = mock_range(20, Price(95.0), Price(105.0))
        assert classify_regime(data) is Regime::Ranging { .. }
    }

    test "volatile": {
        let data = mock_volatile(20, Price(100.0), 5.0)
        let result = classify_regime(data)
        assert result is Regime::Volatile { atr > 3.0 }
    }
}

// Dependency injection
fn main() -> Result<(), Error> {
    let feed: impl MarketDataFeed = BinanceFeed::new(config.api_key)
    let data = feed.fetch("BTCUSDT", yesterday(), now())?
    let regime = classify_regime(data)
    print("Market regime: {regime}")
    Ok(())
}
```

### Compilation Pipeline

```
spec.khold
    │
    ▼
┌─────────┐     ┌──────────┐     ┌───────────┐     ┌──────────┐
│  Parse   │────▶│  Analyze  │────▶│  Generate  │────▶│  Output   │
│          │     │           │     │            │     │           │
│ tree-    │     │ type      │     │ pattern    │     │ src/      │
│ sitter   │     │ checker   │     │ solver     │     │ main.rs   │
│ → AST    │     │ → IR      │     │ + AI       │     │ types.rs  │
│          │     │           │     │ → code     │     │ tests/    │
└─────────┘     └──────────┘     └───────────┘     │ adr/      │
                                                   └──────────┘
```

### Code Generation Output

For the spec above, the compiler generates:

```
src/
├── main.rs              From fn main() spec
├── types.rs             From type OHLCV, enum Regime
├── market_data.rs       From interface MarketDataFeed
├── classify_regime.rs   From fn classify_regime spec
└── error.rs             From FeedError, Error types

tests/
└── adr/
    └── classify_regime_tests.rs  From test blocks in spec
```

### Fallback to AI

When the solver finds no pattern for `classify_regime`, the typed IR is sent to
an LLM with the prompt:

```
Generate Rust code for this specification:

FUNCTION: classify_regime
SIGNATURE: fn(Vec<OHLCV>) -> Regime
RETURN TYPE: Regime (enum with Trending, Ranging, Volatile)
CONSTRAINTS: O(n) time, O(1) space, pure function
TESTS MUST PASS: [3 test cases attached]

The tests are also attached as Rust code. Your implementation
MUST compile and pass all three tests.

Output only the function body for classify_regime, no explanation.
```

This is fundamentally different from v1.0's approach: the AI receives a
structured, typed specification with executable tests — not a natural-language
task description. The output is validated immediately.

## Consequences

### Positive

- Specifications are machine-checkable: type errors, missing tests, and
  constraint violations are caught before code generation.
- Reproducible builds: same spec + same pattern DB → same code output.
- Tests are part of the spec, not an afterthought — they're written before
  the implementation exists.
- Self-documenting: the `.khold` file IS the design document.
- AI context is precise and constrained — no ambiguity to hallucinate on.

### Negative

- Learning curve: developers must learn the DSL syntax.
- Expressiveness ceiling: some designs are hard to express as types +
  constraints (e.g., "the UI should feel snappy").
- Template maintenance: the code generator needs templates for each language
  target (Rust struct gen, Go struct gen, etc.).
- Spec-first workflow: you must write the spec before you get code. This feels
  slower than just writing the code directly for simple cases.

### Risks

- **Spec-code impedance mismatch**: Features that are easy to code but hard to
  specify (e.g., complex state machines, async error handling) may be bypassed.
  Mitigation: allow `extern` blocks that hand-wave implementation.
- **DSL versioning**: As the DSL evolves, old specs must remain compilable.
  Mitigation: version field in project declaration, compiler supports N-1.
- **Over-specification**: Spending more time on the spec than the code would
  have taken. Mitigation: make the DSL concise; a 20-line function should take
  5 lines of spec.

### Scope

V2.0 starts with Rust code generation only. Go and Python generators are
planned for Phase 2. The DSL itself is language-agnostic — the same `.khold`
file can target multiple languages.

### Relationship to Other ADRs

- **ADR-0002 (Pipeline)**: DSL compilation is the primary pipeline path.
- **ADR-0004 (Executable Specs)**: Test blocks in `.khold` are the executable
  specifications.
- **ADR-0006 (Solver)**: The constraint solver matches spec signatures to
  implementation patterns.

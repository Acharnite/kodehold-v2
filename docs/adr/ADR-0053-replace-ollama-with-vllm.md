# ADR-0053: Hybrid Embedding Strategy — sentence-transformers + Ollama

## Status

**Accepted** — 2026-07-09

## Context

The krypto-agent project requires simultaneous LLM generation and embedding capabilities for RAG pipelines. Ollama cannot run inference (qwen3.5:9b) and embedding (bge-m3) concurrently due to VRAM limitations on the available hardware (GTX 1080Ti with 11GB VRAM).

### Problem
- Ollama blocks embedding requests when LLM is busy processing generation requests
- This causes RAG pipeline timeouts and degraded user experience
- Ollama's single-process architecture cannot serve both model types concurrently

### Why bge-m3 works on CPU
- **Architecture:** bge-m3 uses XLMRobertaModel (encoder-only, Transformers-compatible)
- **Size:** ~567M parameters, ~1.5GB model weights
- **CPU Performance:** Embedding short texts (<512 tokens) completes in <100ms on modern CPU
- **No VRAM Required:** Runs entirely on CPU, zero GPU memory consumption
- **Installation:** `pip install sentence-transformers` — no special compilation needed

### Hardware Constraints
- **GPU:** NVIDIA GTX 1080Ti (Pascal architecture, 11GB VRAM)
- **LLM Model:** qwen3.5:9b (~5.5-7.5GB VRAM)
- **Embedding Model:** bge-m3 (~1.5GB VRAM)
- **Overhead:** ~0.5GB for CUDA context and buffers
- **Total Required:** 7.5-8.5GB (within 11GB limit, but Ollama cannot manage dual-model efficiently)

## Decision

Adopt a **hybrid embedding strategy**: use **sentence-transformers on CPU** for embeddings, keep **Ollama for LLM inference** (qwen3.5:9b).

### Rationale
- sentence-transformers runs bge-m3 on CPU — no VRAM conflict with qwen3.5:9b
- Ollama's blocking issue is solved: embedding requests no longer wait for LLM
- Minimal dependencies: just `pip install sentence-transformers`
- No need for vLLM dual-instance complexity

### Architecture

```
┌─────────────────────────────────────────────────┐
│                   krypto-agent                   │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌─────────────┐    ┌─────────────────────┐    │
│  │   Ollama    │    │ sentence-transformers│    │
│  │  (port 11434)│    │     (CPU only)      │    │
│  │  qwen3.5:9b │    │     bge-m3          │    │
│  │  (GPU)      │    │     (~567M params)  │    │
│  └─────────────┘    └─────────────────────┘    │
│         │                    │                  │
│         └────────┬───────────┘                  │
│                  │                              │
│         OpenAI-compatible API                   │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Key Benefits
- **No VRAM Conflict:** Embeddings run on CPU, LLM uses full GPU
- **No Ollama Blocking:** Embedding requests served independently of LLM
- **Simple Setup:** One pip package, no process management needed
- **Fast Enough:** <100ms embedding latency for short texts on CPU
- **Zero GPU Memory for Embeddings:** Leaves all 11GB VRAM for qwen3.5:9b

### Implementation Details

1. **Install sentence-transformers:** `pip install sentence-transformers`
2. **Load model in code:**
   ```python
   from sentence_transformers import SentenceTransformer
   model = SentenceTransformer("BAAI/bge-m3")
   embeddings = model.encode(["query text", "document text"])
   ```
3. **Keep Ollama for LLM:** Ollama continues serving qwen3.5:9b on port 11434
4. **Update rag_memory.py:** Replace Ollama embedding calls with sentence-transformers
5. **No config changes needed:** Embedding endpoint becomes local Python call, not HTTP

## Consequences

### Positive
- ✅ No VRAM conflict — embeddings on CPU, LLM on GPU
- ✅ Ollama blocking issue solved
- ✅ Minimal setup — just one pip install
- ✅ Fast embedding latency (<100ms for short texts)
- ✅ All GPU memory available for LLM (qwen3.5:9b gets full 11GB)

### Negative
- ❌ CPU-only embeddings may be slower for batch processing (1000+ docs)
- ❌ Additional Python dependency (sentence-transformers)
- ❌ No health check endpoint (in-process library, not a service)

### Risks
- **Batch Performance:** If embedding large corpora (>10K docs), CPU may bottleneck — acceptable for RAG query-time embeddings
- **Model Compatibility:** bge-m3 works well with sentence-transformers, verified via XLMRobertaModel support

## Alternatives Considered

1. **vLLM Dual-Instance:** Run two vLLM instances (LLM on port 8000, embeddings on port 8001)
   - Rejected: Overkill for embeddings, adds process management complexity, requires VRAM splitting

2. **Ollama with Model Swapping:** Load/unload models on demand
   - Rejected: Adds latency, not suitable for concurrent requests

3. **Cloud-based embedding API:** Use external service for embeddings
   - Rejected: Adds network latency, cost, and dependency

4. **✅ sentence-transformers on CPU (CHOSEN):** Local embedding library
   - Selected: Simple, fast enough, no VRAM conflict, minimal dependencies

## Review Notes

- **2026-07-09:** Updated to reflect sentence-transformers solution (replacing vLLM dual-instance plan)
- **Previous review:** Hardware model, VRAM calculations, test counts verified

## References

- sentence-transformers Documentation: https://www.sbert.net/
- bge-m3 Model Card: https://huggingface.co/BAAI/bge-m3
- Ollama: https://ollama.com/
- Design Doc Section 8.1: Bring Your Own Model

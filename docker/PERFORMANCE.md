# Docker Performance Notes

## Summary

IMPALA training inside Docker on Windows (Docker Desktop / WSL2)
previously collapsed at 4+ actors. The root cause was PyTorch's
OpenMP intra-op threading — each actor spawned multiple threads,
causing catastrophic oversubscription. Setting `OMP_NUM_THREADS=1`
(now baked into the Dockerfile) fixes the collapse and improves
throughput at all actor counts.

## Benchmarks (2026-03-30, Desktop: Ryzen 9 5900X, RTX 4090)

### Actor scaling with shared model weights

Each actor runs VecGauntletEnv (16 envs) + CPU model inference,
sharing model weights via `torch.share_memory_()`.

| Actors | Docker (sps) | Native (sps) | Ratio |
|--------|-------------|--------------|-------|
| 1      | 6,032       | 5,217        | 1.16x (Docker faster) |
| 2      | 8,033       | 7,568        | 1.06x (Docker faster) |
| 4      | 292         | 9,493        | 0.03x (Docker 32x slower) |

### Component benchmarks (all show Docker >= native)

| Component | Docker | Native |
|-----------|--------|--------|
| VecGauntletEnv stepping (16 envs) | 43,211 steps/sec | 25,252 steps/sec |
| Single actor loop (inference + stepping) | 6,838 steps/sec | 6,543 steps/sec |
| mp.Array shared memory throughput | 6,088-9,016 MB/s | 1,435-4,642 MB/s |
| multiprocessing spawn (numpy work) | 36,678 iter/sec (8w) | 28,656 iter/sec (8w) |

### Full IMPALA training (4 actors, 16 envs each)

| Metric | Docker | Native |
|--------|--------|--------|
| Steps/sec | 286-295 | 4,806 |
| Dequeue % | 87% | 47% |

## Root cause: OMP thread oversubscription

The 4+ actor collapse was caused by PyTorch's OpenMP intra-op
threading, not `torch.share_memory_()` as originally suspected.
Each actor process spawns multiple OpenMP threads for CPU matrix
ops. With 4+ actors, the total thread count far exceeds available
cores, and the WSL2 kernel thrashes on scheduling.

Setting `OMP_NUM_THREADS=1` forces single-threaded matrix ops per
process, which is optimal when parallelism comes from multiple
actor processes rather than threads within one process.

### Training with OMP_NUM_THREADS=1 (torch actors, 2026-03-31)

| Config                  | Steps/sec | Time (500K steps) |
|-------------------------|-----------|-------------------|
| 2 actors, no OMP        | 1,861     | 269.2s            |
| 2 actors, OMP=1         | 2,576     | 194.4s (1.4x)     |
| 4 actors, OMP=1         | 3,733     | 134.0s (2.0x)     |
| 4 actors, no OMP        | **hung**  | timed out at 5min |

The Dockerfile now sets `ENV OMP_NUM_THREADS=1 MKL_NUM_THREADS=1`.

### Numpy actors (2026-04-01, Desktop: Ryzen 9 5900X, RTX 4090)

Actors can use pure-numpy inference instead of PyTorch
(`--actor-backend numpy`, now the default). This eliminates
PyTorch's per-call dispatch overhead for the small DeckModel
(302K params, batch_size=1 per env).

**Actor count sweep** (16 envs/actor, numpy, OMP=1, 500K steps):

| Actors | Steps/sec | Time (s) |
|--------|-----------|----------|
| 2      | 5,032     | 99.4     |
| **4**  | **8,114** | **61.7** |
| 6      | 6,888     | 72.7     |
| 8      | 7,579     | 66.0     |

**Envs-per-actor sweep** (4 actors, numpy, OMP=1, 500K steps):

| Envs/actor | Steps/sec | Time (s) |
|------------|-----------|----------|
| 8          | 8,017     | 62.4     |
| **16**     | **8,114** | **61.7** |
| 32         | 7,692     | 65.1     |

**Optimal: 4 actors × 16 envs** at ~8,100 sps.

Compared to the original configuration (torch actors, 2 actors, no
OMP): 1,861 → 8,114 sps = **4.4x total training speedup**. A
5M-step combat-only run takes ~10 minutes instead of ~45.

## Additional finding: --cpus flag

Do **not** use `--cpus` on Docker Desktop / WSL2. The CFS bandwidth
quota enforcement adds overhead that halves throughput for this
workload (1,500 sps with `--cpus=10` vs 4,000 sps without). Omit
the flag and let Docker use all available CPUs.

## Recommendation

Use **4 actors with 16 envs each** and the default numpy actor
backend. The Dockerfile already sets `OMP_NUM_THREADS=1`. Do not
use `--cpus`.

## Configuration

```bash
# Recommended Docker training command (all defaults)
docker run --rm --gpus all --shm-size=8g \
  -v $(pwd)/runs:/output \
  cyberloop-train \
  --subclass dueling \
  --checkpoint-dir /output

# Explicit (equivalent to defaults)
docker run --rm --gpus all --shm-size=8g \
  -v $(pwd)/runs:/output \
  cyberloop-train \
  --subclass dueling \
  --n-actors 4 \
  --envs-per-actor 16 \
  --actor-backend numpy \
  --checkpoint-dir /output

# Fall back to torch actors for comparison
docker run --rm --gpus all --shm-size=8g \
  -v $(pwd)/runs:/output \
  cyberloop-train \
  --subclass dueling \
  --actor-backend torch \
  --checkpoint-dir /output
```

---

## Evaluation Performance (2026-03-31)

### Summary

The eval container (`cyberloop-eval-rl`) uses `eval_checkpoint.py` to
run greedy episodes in parallel via `ProcessPoolExecutor`. Three
optimizations yield a combined **~9x speedup**:

1. `OMP_NUM_THREADS=1` — prevents PyTorch thread oversubscription
2. Auto worker count — uses all available CPUs instead of hardcoded 2
3. Numpy inference — pure-numpy forward pass replaces PyTorch

### Eval benchmarks (100 episodes, defense, ~35 mean fights)

| Config                            | Time (s) | ep/s  | vs original |
|-----------------------------------|----------|-------|-------------|
| Original (torch 2w, no OMP)      | 66.8     | 1.50  | 1.0x        |
| Torch + OMP=1 + auto workers     | 17.0     | 5.88  | 3.9x        |
| **Numpy + OMP=1 + auto workers** | **7.5**  | 13.33 | **8.9x**    |

200 episodes: **10.0s** with numpy (vs ~134s estimated original).

### Critical: OMP_NUM_THREADS=1

Without `OMP_NUM_THREADS=1`, Docker eval with 4+ worker processes
hangs indefinitely. PyTorch spawns internal OpenMP threads per
worker; with 4+ workers the WSL2 kernel thrashes on thread
scheduling. The Dockerfile now sets this via
`ENV OMP_NUM_THREADS=1 MKL_NUM_THREADS=1`.

### Worker scaling (Docker, OMP_NUM_THREADS=1, torch backend)

| Workers | Time (s) | ep/s  |
|---------|----------|-------|
| 1       | 89.8     | 1.11  |
| 2       | 43.2     | 2.31  |
| 4       | 22.9     | 4.37  |
| 8       | 15.1     | 6.62  |
| 12      | 15.3     | 6.54  |

Optimal: 8 workers on 12-core Ryzen 9 5900X. Saturates at 8;
12 shows no further gain.

### Per-step profiling (numpy, single episode)

| Component         | Time   | % of total |
|-------------------|--------|------------|
| numpy forward     | 0.196s | 56.3%      |
| env.step_features | 0.084s | 24.1%      |
| _pad_obs          | 0.067s | 19.2%      |
| other             | 0.002s | 0.5%       |

Compare to PyTorch where model.forward() was 82.3%. With numpy
inference, the Rust engine and observation padding become
significant fractions. Further optimization is possible by
skipping observation padding (~13% gain) but diminishing returns.

### Docker vs native

Docker (WSL2) is **2.7x faster** than native Windows for
single-worker torch eval due to Linux `fork()` vs Windows `spawn()`
for multiprocessing.

### Container startup overhead

~2.5s per container launch. Negligible for 20+ episode runs.

### Eval configuration

```bash
# Default: numpy backend, auto worker count, OMP=1 baked in
docker run --rm \
  -v /path/to/checkpoint.pt:/input/checkpoint.pt:ro \
  cyberloop-eval-rl \
  --checkpoint /input/checkpoint.pt --subclass dueling --episodes 200

# Torch backend for validation
docker run --rm \
  -v /path/to/checkpoint.pt:/input/checkpoint.pt:ro \
  cyberloop-eval-rl \
  --checkpoint /input/checkpoint.pt --subclass dueling --episodes 200 \
  --backend torch
```

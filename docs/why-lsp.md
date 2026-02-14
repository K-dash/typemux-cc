# Why LSP Matters for Claude Code in Monorepos

> Real-world benchmarks comparing text search (ripgrep) vs LSP (ty backend via typemux-cc) on a production Python monorepo.

## TL;DR

In monorepos, **text search produces structural false positives** that LSP eliminates at the type-system level. This directly impacts Claude Code's ability to make correct code changes.

| Task | Text Search | LSP |
|------|-------------|-----|
| Reference lookup | 174KB output, 5 conflicting definitions mixed | 99 references, zero false positives |
| Dependency navigation | 36 candidates, manual cross-referencing needed | 1-hop `goToDefinition` |
| Rename impact analysis | ~80 hits, 2 different classes mixed | 54 references, exact target only |

## Test Environment

| Item | Value |
|------|-------|
| OS | macOS (arm64 / Apple Silicon) |
| Python | 3.14 |
| LSP backend | ty (via typemux-cc) |
| Text search | ripgrep (`\b` word-boundary regex) |

### Codebase Under Test

| Item | Value |
|------|-------|
| Structure | Python monorepo (DDD + Hexagonal Architecture) |
| Projects | Multiple Python backends + frontend + infrastructure |
| Python files | ~6,500 |
| Python LOC | ~780,000 |
| Key characteristic | Identical type names independently defined across projects |

### Task Selection Criteria

Three scenarios where text search **structurally** struggles in monorepos:

1. Same-named types scattered across projects → false positives in reference lookup
2. Same-named interfaces within a single project (36 instances) → costly dependency identification
3. Same-named classes across layers (domain vs ORM) → risky rename operations

---

## Task 1: Reference Lookup — Find All Usages of `OrderId`

**Goal**: Find all references to `OrderId = NewType("OrderId", int)` defined in `project-alpha/domain/order/ids.py`.

### Text Search Results

| Scope | Hits | Problem |
|-------|------|---------|
| Entire monorepo `\bOrderId\b` | 174KB (output truncated) | Results from all projects mixed together |
| project-alpha only | 97 lines | protobuf-generated `entity_pb2.OrderId` included as false positive |

**Root cause**: 5 independent definitions of `OrderId` exist across the monorepo:

| Location | Type |
|----------|------|
| project-alpha/domain/order/ids.py | `NewType("OrderId", int)` |
| project-beta/domain/order/order.py | `NewType("OrderId", int)` |
| project-gamma/domain/order/order.py | `NewType("OrderId", int)` |
| project-delta/domain/order/order.py | `NewType("OrderId", int)` |
| entity_pb2.OrderId | protobuf generated class |

grep cannot distinguish between these.

### LSP `findReferences` Results

- **99 references / 35 files** (all within project-alpha)
- Only references to `project-alpha.domain.order.ids.OrderId` returned
- protobuf `entity_pb2.OrderId` excluded
- Other projects' same-named types excluded

### Comparison

| Aspect | Text Search | LSP |
|--------|-------------|-----|
| Result volume | 174KB (entire monorepo) | 99 refs / 35 files |
| False positives | 5 different definitions mixed | Exact definition only |
| Post-processing | Manual import-tracing to identify definition source | None required |

---

## Task 2: Dependency Navigation — Understand an Unfamiliar Use Case

**Goal**: Understand the entry point and dependencies of `project-beta/usecase/process_payment/usecase.py`.

### Text Search Approach

1. grep import statements → 8 dependency modules identified
2. `__init__` signature uses `Repository` → "Which Repository?"
3. grep `class Repository` in project-beta/domain → **36 hits**
4. Manual cross-referencing between imports and grep results needed to identify the 6 relevant Repositories

36 `class Repository(ABC):` definitions exist under project-beta/domain alone. grep cannot determine which ones are relevant.

### LSP Approach

1. `documentSymbol` → class `Usecase` + methods + helpers identified structurally (1 API call)
2. `hover` → return types confirmed instantly
3. `goToDefinition` on a domain dependency → jumped directly to `project-beta/domain/payment_policy/__init__.py` (zero filtering from 36 candidates)
4. `goToDefinition` on an infrastructure service → jumped directly to the correct module

### Comparison

| Aspect | Text Search | LSP |
|--------|-------------|-----|
| Structure comprehension | Read entire file, parse mentally | `documentSymbol` — 1 call |
| Dependency identification | grep 36 hits → cross-reference with imports | `goToDefinition` — 1 hop |
| Steps required | grep → Read → grep → manual matching | documentSymbol → goToDefinition |

---

## Task 3: Rename Impact Analysis — `PaymentNotification`

**Goal**: Identify all files/locations that need changes if renaming `PaymentNotification` to `Notification`.

### Text Search Results

- **~80 lines** matched
- Two different classes with the same name are mixed:

| Class | Location | Purpose |
|-------|----------|---------|
| Domain entity | project-alpha/domain/.../notification.py | Business logic |
| SQLAlchemy model | project-alpha/models/payment_notification.py | ORM / database |

**Typical false positives**:
```python
# These are SQLAlchemy model references — NOT the domain entity being renamed
query = select(PaymentNotification.id)
query = query.where(PaymentNotification.user_id.in_(...))
```

In converter files, **both are imported in the same file**:
```python
from ...notification import PaymentNotification          # domain
from project_alpha.models... import PaymentNotification as ...Model  # ORM
```

### LSP `findReferences` Results

- **54 references / 13 files** (domain entity only)
- SQLAlchemy model references completely excluded
- Exact set of locations that need renaming

### Comparison

| Aspect | Text Search | LSP |
|--------|-------------|-----|
| Hit count | ~80 lines | 54 refs / 13 files |
| Same-name class mixing | Domain entity + SQLAlchemy model mixed | Domain entity only |
| Risk of incorrect rename | May accidentally modify ORM layer | Zero |

---

## Why This Matters for Claude Code

Claude Code uses LSP tools (`goToDefinition`, `findReferences`, `hover`, `documentSymbol`) to understand code before making changes. In monorepos:

- **Text search false positives mislead the agent** — Claude Code may modify the wrong `OrderId`, the wrong `Repository`, or the wrong `PaymentNotification`
- **LSP eliminates ambiguity at the type-system level** — each reference is traced back to its exact definition, across files and projects
- **Fewer tool calls, less context consumed** — LSP returns precise results in 1-2 calls vs. multiple rounds of grep + Read + manual filtering

typemux-cc enables this by keeping the correct LSP backend alive for each `.venv` in the monorepo, so Claude Code always gets type-accurate results regardless of which project it's working in.

# game/src/witty_phrase_generator/

## Responsibility
Procedural phrase generation for game naming. Creates alliterative or constrained-length word combinations from embedded wordlists (intensifiers, adjectives, nouns). Used exclusively by `Game::default()` to generate unique, memorable game titles like "mighty-purple-dragon" or "ultra-brilliant-phoenix".

## Design Patterns

### **Builder Pattern**
- `WPGen` struct encapsulates configuration (wordlists) and RNG state
- Methods chain parameters for phrase generation (`with_words()`, `generic()`, `with_phrasewise_alliteration()`)
- Constructor `new()` loads embedded wordlists from static string slices

### **Strategy Pattern (via Parameters)**
- Different generation strategies selected via method choice:
  - `with_words(n)` - Simple random selection (fast, no constraints)
  - `generic(...)` - Length/alliteration constraints (complex, backtracking)
  - `with_phrasewise_alliteration(...)` - Per-phrase alliteration (retry loop)
- Constraint parameters: `len_min`, `len_max`, `word_len_max`, `start_char`

### **Backtracking Algorithm**
- `generate_backtracking()` recursively searches for valid word combinations
- Prunes search space using word length upper bounds
- Returns `Option<Vec<&'static str>>` - None if constraints unsatisfiable

### **Interior Mutability**
- `rng: RefCell<ThreadRng>` allows mutation through `&self` methods
- Necessary because `choose()` requires mutable RNG access
- Single-threaded design (not `Sync`) - safe for game engine

### **Embedded Resources**
- Wordlists compiled into binary via `include_str!()` macro
- Static lifetime references (`&'static str`) avoid allocations
- Files: `intensifiers.txt` (194KB), `adjectives.txt` (45KB), `nouns.txt` (8KB)

## Data & Control Flow

### **Initialization**
```
WPGen::new()
  ├─> include_str!("intensifiers.txt").lines() → Vec<&'static str> [~4000 words]
  ├─> include_str!("adjectives.txt").lines()   → Vec<&'static str> [~1500 words]
  ├─> include_str!("nouns.txt").lines()        → Vec<&'static str> [~500 words]
  └─> ThreadRng::default() → RefCell<ThreadRng>
```

### **Simple Generation** (`with_words(n)`)
```
with_words(3) → ["ultra", "brilliant", "phoenix"]
  ├─> if n > 2: choose from intensifiers → ret[0]
  ├─> if n > 1: choose from adjectives   → ret[1]
  ├─> if n > 0: choose from nouns        → ret[2]
  └─> if n > 3: choose second noun       → ret[3]  [max 4 words]
```

### **Constrained Generation** (`generic(...)`)
```
generic(words=3, count=5, len_min=10, len_max=20, ...)
  ├─> Filter wordlists:
  │     ├─> if start_char: retain words starting with 'c'
  │     ├─> retain words with length <= word_len_max
  │     ├─> shuffle() [randomize order]
  │     └─> sort_by(|a,b| a.len().cmp(b.len())) [enable length matching]
  ├─> For each of 5 phrases:
  │     └─> generate_backtracking(10, 20, 1, dict, format):
  │           ├─> Binary search upper_bound (words <= len_max)
  │           ├─> choose_multiple() from pool [random sampling]
  │           ├─> For each candidate:
  │           │     ├─> if last word: check len >= len_min → return
  │           │     └─> else: recurse with reduced budget:
  │           │           generate_backtracking(
  │           │             (len_min - selected.len()).max(0),
  │           │             len_max - selected.len(),
  │           │             depth+1, ...
  │           │           )
  │           └─> if recursion succeeds: return Some(vec)
  └─> Return Some(Vec<Vec<&str>>) or None if any phrase fails
```

### **Format Creation**
- `create_format(words) -> Vec<usize>` maps word count to dictionary indices:
  - 1 word: [0, 3] → noun
  - 2 words: [0, 2, 3] → adjective + noun
  - 3 words: [0, 1, 2, 3] → intensifier + adjective + noun
  - 4 words: [0, 1, 2, 3] → intensifier + adjective + noun + noun
- Indices: 0=unused, 1=intensifiers, 2=adjectives, 3=nouns

### **Alliteration Strategy**
```
with_phrasewise_alliteration(words=3, count=5, ...)
  └─> For each of 5 phrases:
        └─> loop:
              ├─> Random char ∈ [a-z]
              ├─> generic(words, 1, ..., Some(char))
              └─> if Some(phrase): break [retry until success]
```

## Integration Points

### **Consumed By**
- **games.rs**: 
  - `Game::default()` calls `WPGen::new().with_words(3).unwrap()`
  - Joins words with hyphens: `vec.join("-")` → "mighty-purple-dragon"
  - Used for auto-generated game names when no name provided

### **Depends On**
- **External Crates**:
  - `rand` - RNG for word selection (`ThreadRng`, `SliceRandom`, `Rng`)
  - `std::cell::RefCell` - Interior mutability for RNG

### **Data Files** (embedded at compile time)
- **intensifiers.txt**: 194KB, ~4000 words (e.g., "ultra", "mega", "supremely")
- **adjectives.txt**: 45KB, ~1500 words (e.g., "brilliant", "crimson", "swift")
- **nouns.txt**: 8KB, ~500 words (e.g., "phoenix", "dragon", "tiger")

### **Memory Footprint**
- `WPGen` struct: ~247KB (wordlists + RNG state)
- Created once per game initialization, short-lived
- Wordlists are `&'static str` refs (no heap allocation for word data)

## Key Files

### **mod.rs** (205 lines)
- **Purpose**: Phrase generation engine with constraint solving
- **Key Struct**: `WPGen`
  - Fields:
    - `rng: RefCell<ThreadRng>` - Mutable RNG state
    - `words_intensifiers: Vec<&'static str>` - Embedded word list 1
    - `words_adjectives: Vec<&'static str>` - Embedded word list 2
    - `words_nouns: Vec<&'static str>` - Embedded word list 3
- **Public API**:
  - `new() -> WPGen` - Loads wordlists from embedded files
  - `with_words(n) -> Option<Vec<&'static str>>` - Simple random (1-4 words)
  - `generic(words, count, len_min, len_max, word_len_max, start_char) -> Option<Vec<Vec<&str>>>` - Constrained generation
  - `with_phrasewise_alliteration(words, count, len_min, len_max, word_len_max) -> Option<Vec<Vec<&str>>>` - Per-phrase alliteration
- **Private Helpers**:
  - `create_format(words) -> Vec<usize>` - Maps word count to dictionary access pattern
  - `generate_backtracking(len_min, len_max, depth, dict, format) -> Option<Vec<&'static str>>` - Recursive constraint solver
- **Algorithm Details**:
  - Backtracking uses binary search for length upper bounds (commented out, replaced with linear scan)
  - `choose_multiple()` randomizes candidate order for diversity
  - `shuffle()` + `sort_by(len)` creates stable-randomized length ordering
- **Design Notes**:
  - `#[allow(dead_code)]` on impl block - methods unused except `with_words()`
  - No tests in mod.rs (functionality covered by name_generator tests)
  - Binary search optimization commented out ("TODO: is binary search even faster on such short wordlists?")
  - Format vector includes unused index 0 for alignment (simplifies indexing logic)

### **intensifiers.txt** (194KB, ~4000 words)
- **Purpose**: Emphatic prefix words
- **Examples**: "ultra", "mega", "super", "hyper", "supremely", "incredibly"
- **Usage**: Optional first word in 3-4 word phrases
- **Source**: Unknown (possibly scraped from thesaurus/word lists)

### **adjectives.txt** (45KB, ~1500 words)
- **Purpose**: Descriptive modifiers
- **Examples**: "brilliant", "crimson", "swift", "mighty", "fearless"
- **Usage**: Second word in 3+ word phrases, or first in 2-word phrases
- **Source**: Unknown (possibly scraped from thesaurus/word lists)

### **nouns.txt** (8KB, ~500 words)
- **Purpose**: Concrete subjects
- **Examples**: "phoenix", "dragon", "tiger", "wolf", "eagle", "storm"
- **Usage**: Final word in all phrases (always present)
- **Source**: Unknown (possibly curated for heroic/dramatic themes)

## Notes
- **Single Use Case**: Only called once per game initialization - not performance-critical
- **Unused Complexity**: `generic()` and `with_phrasewise_alliteration()` never called in codebase - `with_words()` sufficient for current needs
- **No Validation**: Wordlists assumed well-formed (one word per line, no empty lines)
- **Alliteration Bias**: Retry loop in `with_phrasewise_alliteration()` may hang if no valid words for chosen letter
- **Binary Search Trade-off**: Linear scan used instead of binary search - wordlists small enough (max 4000 entries) that O(n) vs O(log n) negligible
- **Thread Safety**: `RefCell` makes `WPGen` not `Sync` - single-threaded only
- **Future Work**: Add phrase validation (profanity filter), curate wordlists for thematic consistency, expose `generic()` API for user-customizable game names

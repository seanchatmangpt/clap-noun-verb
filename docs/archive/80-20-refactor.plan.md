# ggen 80/20 Refactoring Plan

## Principle: Focus on Maximum Impact with Minimum Effort

This plan targets the **20% of changes that deliver 80% of the value**:

1. **CLI Migration** (highest user-facing impact)
2. **Separation of Concerns** (biggest maintainability gain)
3. **Module Boundaries** (eliminates coupling issues)
4. **Core Abstraction** (enables future flexibility)

---

## Phase 1: CLI Migration (Week 1-2) - 40% of Value

### Priority: HIGHEST - User-facing, immediate benefits

**Goal**: Migrate all 13 CLI commands to clap-noun-verb v3.0.0

**Why 80/20**: This is the **most visible** change with **immediate** developer experience improvements.

**Approach**:

1. **Migrate simple commands first** (80% of commands, 20% of complexity):

   - `utils doctor` (proof-of-concept)
   - `utils help-me`
   - `project new`
   - `hook create`

2. **Migrate core commands** (20% of commands, 80% of usage):

   - `ai project` (most used)
   - `ai generate` (most used)
   - `marketplace search` (most used)
   - `template generate` (most used)

3. **Migrate remaining commands**:

   - `ai graph`, `ai sparql`
   - `marketplace install`, `marketplace list`, `marketplace publish`

**Expected Impact**:

- **50% reduction** in CLI boilerplate code
- **100% elimination** of manual command registration
- **Auto-discovery** enables rapid feature addition
- **JSON output** improves automation/scripting

**Key Files**:

- `cli/commands/*.rs` - New command structure
- `cli/mod.rs` - Entry point with auto-discovery
- `cli/handlers/*.rs` - Business logic separation

---

## Phase 2: Separation of Concerns (Week 2-3) - 30% of Value

### Priority: HIGH - Eliminates 80% of maintenance pain

**Goal**: Strict separation between CLI layer and business logic layer

**Why 80/20**: This **eliminates 80% of coupling issues** and enables **independent testing/development**.

**Approach**:

1. **Extract all business logic** from CLI handlers:
   ```rust
   // BEFORE: Mixed CLI and business logic
   fn handle_ai_project(args: &ArgMatches) {
       let name = args.get_one::<String>("name").unwrap();  // CLI-specific
       // Business logic here - can't test independently
   }
   
   // AFTER: Separated
   // Business Logic Layer (testable independently)
   pub fn create_project(name: String, rust: bool) -> ProjectOutput {
       // Pure function - testable without CLI
   }
   
   // CLI Layer (delegates only)
   #[verb]
   fn ai_project(name: String, rust: bool) -> Result<ProjectOutput> {
       Ok(create_project(name, rust))  // Delegate to business logic
   }
   ```

2. **Organize business logic by domain**:
   ```
   src/
   ├── domain/
   │   ├── ai/           # AI generation logic
   │   ├── template/     # Template rendering logic
   │   ├── rdf/          # RDF/SPARQL logic
   │   └── marketplace/  # Marketplace logic
   ```

3. **Create clear boundaries**:

   - CLI layer → Business logic (one-way dependency)
   - Business logic → No CLI dependencies
   - Business logic → Pure functions (testable)

**Expected Impact**:

- **80% reduction** in testing complexity (test business logic independently)
- **100% reusability** (business logic usable by API, Web, etc.)
- **Eliminates** circular dependencies
- **Enables** parallel development (CLI and business logic teams)

**Key Files**:

- `src/domain/*/lib.rs` - Domain logic modules
- `cli/handlers/*.rs` - Thin delegation layer
- Extract from existing `ggen-core`, `ggen-ai`, `ggen-marketplace`

---

## Phase 3: Module Boundaries (Week 3-4) - 20% of Value

### Priority: MEDIUM - Prevents future coupling issues

**Goal**: Clear module boundaries with defined interfaces

**Why 80/20**: This **prevents 80% of future architectural debt** with minimal upfront cost.

**Approach**:

1. **Define module interfaces** (20% of effort, prevents 80% of coupling):
   ```rust
   // domain/template/traits.rs
   pub trait TemplateEngine: Send + Sync {
       fn render(&self, template: &str, vars: &Vars) -> Result<String>;
       fn query_sparql(&self, query: &str) -> Result<Vec<Triple>>;
   }
   
   // domain/rdf/traits.rs
   pub trait RdfProcessor: Send + Sync {
       fn query(&self, sparql: &str) -> Result<QueryResult>;
       fn validate(&self, data: &RdfGraph) -> Result<ValidationReport>;
   }
   ```

2. **Implement dependency inversion**:

   - Template engine depends on RDF processor trait (not concrete type)
   - AI integration depends on template engine trait (not concrete type)
   - Enables testing with mocks
   - Enables future flexibility

**Expected Impact**:

- **Prevents** tight coupling between modules
- **Enables** easy testing with mocks
- **Facilitates** future refactoring
- **Improves** modularity and extensibility

**Key Files**:

- `src/domain/*/traits.rs` - Trait definitions
- `src/domain/*/impl.rs` - Trait implementations
- Update existing code to use traits

---

## Phase 4: Core Abstraction (Week 4) - 10% of Value

### Priority: LOW - Enables future extensibility

**Goal**: Extract common patterns into reusable abstractions

**Why 80/20**: This **enables 80% of future extensions** with minimal upfront investment.

**Approach**:

1. **Identify common patterns**:
   - Template generation workflows
   - RDF processing pipelines
   - AI integration patterns
   - Marketplace operations

2. **Extract to abstractions**:
   ```rust
   // Common abstractions
   pub trait Generator: Send + Sync {
       fn generate(&self, input: &Input) -> Result<Output>;
   }
   
   pub trait Processor: Send + Sync {
       fn process(&self, data: &Data) -> Result<ProcessedData>;
   }
   ```

**Expected Impact**:

- **Reduces** code duplication
- **Enables** consistent patterns across modules
- **Facilitates** future extensions
- **Improves** maintainability

---

## Success Metrics

### Code Metrics
- **50%+ reduction** in CLI boilerplate code
- **100% elimination** of manual command registration
- **80% reduction** in testing complexity
- **Zero** circular dependencies

### Performance Metrics
- **<5% performance impact** from refactoring
- **<10% increase** in binary size
- **Maintain** existing performance characteristics

### Quality Metrics
- **100% backward compatibility** with existing commands
- **90%+ test coverage** maintained
- **Zero** breaking changes
- **Improved** developer experience

---

## Implementation Order

1. ✅ **Phase 1: CLI Migration** (Weeks 1-2) - Start here
   - Week 1: Foundation + Simple commands
   - Week 2: Core commands + Remaining commands

2. ⏳ **Phase 2: Separation of Concerns** (Weeks 2-3)
   - Extract business logic
   - Reorganize by domain
   - Create boundaries

3. ⏳ **Phase 3: Module Boundaries** (Weeks 3-4)
   - Define interfaces
   - Implement dependency inversion
   - Update existing code

4. ⏳ **Phase 4: Core Abstraction** (Week 4)
   - Identify patterns
   - Extract abstractions
   - Apply consistently

---

## Next Steps

1. **Create feature branch**: `refactor/80-20-cli-migration`
2. **Set up dependencies**: Add `clap-noun-verb` v3.0.0
3. **Create file structure**: `cli/commands/`, `cli/handlers/`
4. **Implement proof-of-concept**: `utils doctor` command
5. **Validate approach**: Test auto-discovery works
6. **Migrate incrementally**: One command group at a time



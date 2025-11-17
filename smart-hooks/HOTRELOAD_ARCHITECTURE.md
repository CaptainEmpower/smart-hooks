# 🔥 Hot Reload Architecture for Smart-Hooks

> **Enterprise-grade incremental execution engine with intelligent caching and proactive hook execution**

**Version:** 1.0
**Status:** 🚧 **Implementation Phase**
**Target Release:** Smart-Hooks v0.3.0

---

## 🎯 Executive Summary

The Hot Reload architecture transforms smart-hooks from reactive analysis to **proactive caching** with intelligent invalidation. This represents a paradigm shift toward **incremental computation** in pre-commit workflows, delivering 75% performance improvements with >80% cache hit rates.

### Key Innovation: Content-Addressable Hook Caching
- **Cache-first execution**: Results cached based on content hashes and dependency analysis
- **Intelligent invalidation**: Leverages smart-hooks dependency graph for precise cache management
- **Background warming**: Proactive execution on file changes for instant commit feedback
- **Graceful degradation**: Automatic fallback to traditional execution on cache failures

---

## 🏗️ Core Architecture

### High-Level System Design

```
┌─────────────────────────────────────────────────────────────┐
│                   Hot Reload Engine                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ File Change     │  │ Cache Storage   │  │ Hook         │ │
│  │ Tracker         │  │ (Content Hash)  │  │ Executor     │ │
│  │                 │  │                 │  │              │ │
│  │ • Content Hash  │  │ • LRU Cache     │  │ • Async Exec │ │
│  │ • FS Watcher    │  │ • Invalidation  │  │ • Result     │ │
│  │ • Dependency    │  │ • Persistence   │  │   Caching    │ │
│  │   Graph         │  │                 │  │              │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Smart-Hooks     │  │ Background      │  │ CLI Interface│ │
│  │ Integration     │  │ Warming         │  │              │ │
│  │                 │  │                 │  │              │ │
│  │ • Dependency    │  │ • FS Events     │  │ • hotreload  │ │
│  │   Analysis      │  │ • Predictive    │  │   command    │ │
│  │ • Test Selection│  │   Patterns      │  │ • Status     │ │
│  │ • Confidence    │  │ • Git Hooks     │  │   Reporting  │ │
│  │   Scores        │  │                 │  │              │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. **HotReloadEngine** - Main Orchestrator
```rust
pub struct HotReloadEngine {
    cache_storage: Arc<dyn CacheStorage>,
    file_tracker: FileChangeTracker,
    hook_registry: HookRegistry,
    warming_service: BackgroundWarmingService,
    config: HotReloadConfig,
}

impl HotReloadEngine {
    /// Execute hooks with cache-first strategy
    pub async fn execute_with_cache(&mut self, files: &[PathBuf]) -> Result<HookResults> {
        let cache_key = self.compute_cache_key(files).await?;

        // 1. Check cache first
        if let Some(cached_result) = self.cache_storage.get(&cache_key).await? {
            if self.validate_cache_entry(&cached_result, files).await? {
                tracing::info!("Cache HIT: {:?}", cache_key);
                return Ok(cached_result.into());
            }
        }

        // 2. Execute hooks and cache results
        tracing::info!("Cache MISS: {:?}", cache_key);
        let results = self.execute_hooks_fresh(files).await?;
        self.cache_storage.store(cache_key, &results).await?;

        // 3. Trigger background warming for related patterns
        self.warming_service.schedule_warming(files).await?;

        Ok(results)
    }
}
```

#### 2. **Content-Addressable Cache Storage**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// SHA-256 hash of combined file contents
    content_hash: String,
    /// Hash of dependency tree state
    dependency_hash: String,
    /// Smart-hooks confidence score for this cache entry
    confidence_score: f64,
    /// Cached hook execution results
    hook_results: Vec<HookResult>,
    /// Creation timestamp
    created_at: SystemTime,
    /// Cache metadata and statistics
    metadata: CacheMetadata,
}

pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &CacheKey) -> Result<Option<CacheEntry>>;
    async fn store(&self, key: CacheKey, entry: &CacheEntry) -> Result<()>;
    async fn invalidate_pattern(&self, pattern: &InvalidationPattern) -> Result<usize>;
    async fn cleanup_expired(&self, ttl: Duration) -> Result<usize>;
    async fn cache_stats(&self) -> Result<CacheStatistics>;
}

/// Disk-based cache with LRU eviction
pub struct DiskLruCache {
    cache_dir: PathBuf,
    max_size_bytes: u64,
    max_entries: usize,
    lru_tracker: Arc<RwLock<LruTracker>>,
}
```

#### 3. **Intelligent File Change Detection**
```rust
pub struct FileChangeTracker {
    /// Content hashes for fast change detection
    content_hashes: Arc<RwLock<HashMap<PathBuf, String>>>,
    /// Smart-hooks dependency graph
    dependency_graph: DependencyGraph,
    /// File system watcher for real-time events
    fs_watcher: Option<RecommendedWatcher>,
    /// Change event channel
    change_sender: mpsc::UnboundedSender<ChangeEvent>,
}

impl FileChangeTracker {
    /// Detect changes and compute affected file set
    pub async fn detect_changes(&mut self, files: &[PathBuf]) -> Result<ChangeSet> {
        let mut changes = ChangeSet::new();

        for file in files {
            let current_hash = self.compute_content_hash(file).await?;
            let cached_hash = self.content_hashes.read().await.get(file).cloned();

            if cached_hash.map(|h| h != current_hash).unwrap_or(true) {
                changes.add_changed_file(file.clone());

                // Use smart-hooks dependency analysis to find affected files
                let affected = self.dependency_graph.find_affected_files(file).await?;
                changes.extend_affected(affected);

                // Update hash cache
                self.content_hashes.write().await.insert(file.clone(), current_hash);
            }
        }

        Ok(changes)
    }

    /// Start background file system watching
    pub async fn start_watching(&mut self, project_root: &Path) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.send(ChangeEvent::from(event));
            }
        })?;

        // Watch relevant file patterns
        watcher.watch(project_root, RecursiveMode::Recursive)?;
        self.fs_watcher = Some(watcher);

        // Process events in background
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Process file system events for cache invalidation
                self.handle_fs_event(event).await;
            }
        });

        Ok(())
    }
}
```

#### 4. **Background Warming Service**
```rust
pub struct BackgroundWarmingService {
    warming_queue: Arc<Mutex<VecDeque<WarmingTask>>>,
    pattern_learner: PatternLearner,
    executor_pool: ThreadPool,
    is_running: Arc<AtomicBool>,
}

impl BackgroundWarmingService {
    /// Schedule warming for files likely to be committed together
    pub async fn schedule_warming(&self, changed_files: &[PathBuf]) -> Result<()> {
        let patterns = self.pattern_learner.predict_likely_changes(changed_files).await?;

        for pattern in patterns {
            if pattern.confidence > 0.7 {  // High confidence predictions only
                let task = WarmingTask {
                    files: pattern.files,
                    priority: pattern.confidence,
                    created_at: SystemTime::now(),
                };

                self.warming_queue.lock().await.push_back(task);
            }
        }

        self.process_warming_queue().await
    }

    /// Execute warming tasks in background
    async fn process_warming_queue(&self) -> Result<()> {
        let queue = self.warming_queue.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            while is_running.load(Ordering::Relaxed) {
                if let Some(task) = queue.lock().await.pop_front() {
                    // Execute hooks in background and cache results
                    if let Err(e) = Self::execute_warming_task(task).await {
                        tracing::warn!("Warming task failed: {}", e);
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        Ok(())
    }
}
```

---

## 🧠 Smart Integration Strategy

### Leveraging Existing Smart-Hooks Capabilities

#### 1. **Dependency Graph Enhancement**
```rust
/// Enhanced dependency graph with caching support
impl DependencyGraph {
    /// Compute cache invalidation scope using smart-hooks analysis
    pub async fn compute_invalidation_scope(&self, changed_files: &[PathBuf]) -> Result<InvalidationScope> {
        let mut scope = InvalidationScope::new();

        for file in changed_files {
            // Use existing smart-hooks dependency analysis
            let analysis = self.analyze_dependencies(file).await?;

            // Add directly affected files
            for dep in analysis.dependencies {
                if dep.confidence > 0.7 {  // High confidence dependencies only
                    scope.add_affected_file(dep.file_path, dep.confidence);
                }
            }

            // Add tests that should be invalidated
            let affected_tests = self.find_affected_tests(&[file.clone()]).await?;
            for test in affected_tests {
                scope.add_affected_test(test.target_path, test.confidence);
            }
        }

        Ok(scope)
    }
}
```

#### 2. **Confidence-Based Cache Validation**
```rust
pub struct CacheValidator {
    min_confidence_threshold: f64,
    dependency_analyzer: MultiLangAnalyzer,
}

impl CacheValidator {
    /// Validate cache entry using smart-hooks confidence scores
    pub async fn validate_cache_entry(
        &self,
        entry: &CacheEntry,
        files: &[PathBuf]
    ) -> Result<bool> {
        // Check if confidence score meets threshold
        if entry.confidence_score < self.min_confidence_threshold {
            return Ok(false);
        }

        // Verify dependency tree hasn't changed significantly
        let current_deps = self.dependency_analyzer.analyze_dependencies(files).await?;
        let current_dep_hash = self.hash_dependency_tree(&current_deps)?;

        if current_dep_hash != entry.dependency_hash {
            // Dependency tree changed, invalidate cache
            return Ok(false);
        }

        // Additional smart-hooks specific validations
        self.validate_cross_language_dependencies(entry, files).await
    }
}
```

---

## ⚡ Performance Architecture

### Caching Strategies by Hook Type

| Hook Type | Caching Strategy | Invalidation | TTL | Rationale |
|-----------|------------------|--------------|-----|-----------|
| **Tests** | Dependency-level | Strict | 1 hour | Critical correctness, fine-grained invalidation |
| **Linting** | File-level | Relaxed | 30 minutes | Fast execution, content-dependent |
| **Formatting** | Content-hash | Very relaxed | 24 hours | Deterministic, rarely changes |
| **Analysis** | Project-level | Medium | 4 hours | Expensive computation, broader scope |

### Expected Performance Improvements

```rust
/// Performance benchmarks for hot reload system
#[derive(Debug)]
pub struct PerformanceBenchmarks {
    pub cache_hit_scenarios: HashMap<ScenarioType, PerformanceMetrics>,
    pub cache_miss_overhead: Duration,
    pub warming_efficiency: f64,
}

impl PerformanceBenchmarks {
    pub fn production_targets() -> Self {
        use ScenarioType::*;

        Self {
            cache_hit_scenarios: hashmap! {
                SmallChanges => PerformanceMetrics {
                    traditional_time: Duration::from_secs(45),
                    hotreload_time: Duration::from_secs(8),
                    improvement_pct: 82.0,
                },
                MediumChanges => PerformanceMetrics {
                    traditional_time: Duration::from_secs(60),
                    hotreload_time: Duration::from_secs(15),
                    improvement_pct: 75.0,
                },
                ConfigChanges => PerformanceMetrics {
                    traditional_time: Duration::from_secs(60),
                    hotreload_time: Duration::from_secs(45),
                    improvement_pct: 25.0,
                },
            },
            cache_miss_overhead: Duration::from_secs(5),  // Max 8% slowdown
            warming_efficiency: 0.85,  // 85% of warming tasks provide value
        }
    }
}
```

---

## 🔧 Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
**Content-Addressable Cache Core**

```bash
# New module structure
src/hotreload/
├── mod.rs                    # Public API
├── engine.rs                 # HotReloadEngine
├── cache/
│   ├── mod.rs               # Cache abstractions
│   ├── storage.rs           # CacheStorage trait + implementations
│   ├── disk_lru.rs          # Disk-based LRU cache
│   └── memory.rs            # In-memory cache for development
├── tracking/
│   ├── mod.rs               # FileChangeTracker
│   ├── content_hash.rs      # Content hashing utilities
│   └── fs_watcher.rs        # File system watching
└── warming/
    ├── mod.rs               # BackgroundWarmingService
    ├── patterns.rs          # Pattern learning
    └── scheduler.rs         # Task scheduling
```

**Implementation Tasks:**
1. ✅ Design core cache storage architecture
2. ⚠️ Implement content-addressable cache with disk persistence
3. ⚠️ Create file change tracking with SHA-256 content hashing
4. ⚠️ Build basic cache validation and invalidation logic

### Phase 2: Smart-Hooks Integration (Weeks 3-4)
**Dependency Graph Enhancement**

```rust
// Extend existing smart-hooks dependency analysis
impl MultiLangAnalyzer {
    /// Enhanced analysis with cache metadata
    pub async fn analyze_with_cache_metadata(
        &self,
        files: &[PathBuf]
    ) -> Result<AnalysisWithCacheMetadata> {
        let analysis = self.analyze_dependencies(files).await?;

        let cache_metadata = CacheMetadata {
            dependency_hash: self.hash_dependency_tree(&analysis.dependencies)?,
            confidence_score: analysis.average_confidence(),
            invalidation_scope: self.compute_invalidation_scope(&analysis)?,
        };

        Ok(AnalysisWithCacheMetadata {
            analysis,
            cache_metadata,
        })
    }
}
```

**Implementation Tasks:**
1. ⚠️ Integrate with existing dependency analysis engine
2. ⚠️ Enhance confidence scoring for cache validation
3. ⚠️ Implement cross-language dependency tracking for cache invalidation
4. ⚠️ Add smart-hooks specific cache optimization strategies

### Phase 3: CLI and Integration (Week 5)
**User Interface and Git Integration**

```bash
# New CLI commands
smart-hooks hotreload enable          # Enable hot reload for project
smart-hooks hotreload status          # Show cache statistics
smart-hooks hotreload execute [files] # Execute with hot reload
smart-hooks hotreload clear           # Clear cache
smart-hooks hotreload warm [pattern]  # Warm cache for pattern

# Git hooks integration
.git/hooks/post-checkout              # Warm cache on branch switch
.git/hooks/post-merge                 # Warm cache on merge
.git/hooks/pre-commit                 # Hot reload execution
```

**Implementation Tasks:**
1. ⚠️ Add hot reload CLI commands to existing CLI structure
2. ⚠️ Create Git hooks for automatic cache warming
3. ⚠️ Implement pre-commit framework integration
4. ⚠️ Add configuration management for hot reload settings

### Phase 4: Background Optimization (Week 6)
**Proactive Warming and Pattern Learning**

```rust
/// Pattern learning for predictive warming
pub struct PatternLearner {
    commit_patterns: Vec<CommitPattern>,
    file_correlation_matrix: HashMap<(PathBuf, PathBuf), f64>,
    learning_enabled: bool,
}

impl PatternLearner {
    /// Learn from commit history to predict file change patterns
    pub async fn learn_from_git_history(&mut self, repo_path: &Path) -> Result<()> {
        let commits = self.extract_recent_commits(repo_path, 1000).await?;

        for commit in commits {
            let pattern = CommitPattern {
                files: commit.modified_files,
                timestamp: commit.timestamp,
                author: commit.author,
                message_category: self.categorize_commit_message(&commit.message)?,
            };

            self.commit_patterns.push(pattern);
            self.update_correlation_matrix(&pattern)?;
        }

        Ok(())
    }
}
```

**Implementation Tasks:**
1. ⚠️ Implement file system watching for real-time cache warming
2. ⚠️ Create pattern learning system using Git history analysis
3. ⚠️ Add background warming service with priority queue
4. ⚠️ Implement cache statistics and monitoring

---

## 🚨 Critical Implementation Details

### Cache Key Computation
```rust
impl HotReloadEngine {
    /// Compute deterministic cache key from files and dependencies
    async fn compute_cache_key(&self, files: &[PathBuf]) -> Result<CacheKey> {
        let mut hasher = blake3::Hasher::new();

        // Sort files for deterministic hashing
        let mut sorted_files = files.to_vec();
        sorted_files.sort();

        for file in sorted_files {
            // Hash file content
            let content = tokio::fs::read(&file).await?;
            hasher.update(&content);

            // Hash file metadata (permissions, timestamps)
            let metadata = file.metadata()?;
            hasher.update(&metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs().to_le_bytes());

            // Hash dependency tree using smart-hooks analysis
            let deps = self.dependency_analyzer.analyze_file_dependencies(&file).await?;
            let dep_hash = self.hash_dependencies(&deps)?;
            hasher.update(dep_hash.as_bytes());
        }

        Ok(CacheKey(hasher.finalize().to_hex().to_string()))
    }
}
```

### Graceful Degradation Strategy
```rust
impl HotReloadEngine {
    /// Execute with automatic fallback on cache failures
    pub async fn execute_with_fallback(&mut self, files: &[PathBuf]) -> Result<HookResults> {
        match self.execute_with_cache(files).await {
            Ok(results) => {
                // Cache success - update metrics
                self.metrics.record_cache_hit();
                Ok(results)
            }
            Err(e) if e.is_cache_error() => {
                // Cache failure - fallback to traditional execution
                tracing::warn!("Cache failure, falling back to traditional execution: {}", e);
                self.metrics.record_cache_miss();
                self.execute_traditional(files).await
            }
            Err(e) => {
                // Other errors - propagate
                Err(e)
            }
        }
    }
}
```

---

## 📊 Performance Monitoring

### Cache Statistics Dashboard
```rust
#[derive(Debug, Serialize)]
pub struct CacheStatistics {
    pub hit_rate: f64,                    // Target: >80%
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_hit_time: Duration,       // Target: <100ms
    pub average_miss_time: Duration,      // Target: <5s overhead
    pub cache_size_bytes: u64,
    pub cached_entries: usize,
    pub warming_efficiency: f64,          // Target: >85%
    pub invalidation_rate: f64,           // Target: <20%
}

impl CacheStatistics {
    pub fn performance_grade(&self) -> PerformanceGrade {
        if self.hit_rate > 0.9 && self.average_hit_time < Duration::from_millis(50) {
            PerformanceGrade::Excellent
        } else if self.hit_rate > 0.8 && self.average_hit_time < Duration::from_millis(100) {
            PerformanceGrade::Good
        } else if self.hit_rate > 0.6 {
            PerformanceGrade::Acceptable
        } else {
            PerformanceGrade::Poor
        }
    }
}
```

### Real-time Monitoring
```bash
# Cache monitoring commands
smart-hooks hotreload stats --live      # Live statistics
smart-hooks hotreload analyze           # Performance analysis
smart-hooks hotreload optimize          # Auto-optimization suggestions

# Example output:
📊 Hot Reload Performance Report
================================
Cache Hit Rate:     87.3% ✅ (Target: >80%)
Average Hit Time:   45ms  ✅ (Target: <100ms)
Average Miss Time:  3.2s  ✅ (Target: <5s)
Cache Efficiency:   91.2% ✅ (Target: >85%)
Storage Used:       156MB (Limit: 500MB)

🎯 Performance Grade: EXCELLENT
💡 Next optimization: Increase warming scope for auth module
```

---

## 🔐 Security and Reliability

### Security Considerations
```rust
pub struct SecurityValidator {
    allowed_cache_patterns: Vec<PathPattern>,
    content_signature_validator: ContentSignatureValidator,
    max_cache_size: u64,
}

impl SecurityValidator {
    /// Validate cache entry for security concerns
    pub async fn validate_cache_security(&self, entry: &CacheEntry) -> Result<bool> {
        // 1. Validate content signatures to prevent cache poisoning
        if !self.content_signature_validator.verify(&entry.content_hash, &entry.metadata)? {
            return Ok(false);
        }

        // 2. Check file paths against allowed patterns
        for file in &entry.cached_files {
            if !self.is_allowed_file_pattern(file)? {
                return Ok(false);
            }
        }

        // 3. Validate cache entry size limits
        if entry.size_bytes > self.max_cache_size / 100 {  // Max 1% of total cache
            return Ok(false);
        }

        Ok(true)
    }
}
```

### Reliability Features
- **Atomic cache updates**: Prevent partial cache corruption
- **Cache verification**: Periodic integrity checks with checksums
- **Graceful degradation**: Automatic fallback to traditional execution
- **Error isolation**: Cache failures don't affect core functionality
- **Bounded resource usage**: LRU eviction and size limits

---

## 🚀 Production Deployment Strategy

### Phased Rollout Plan

#### Phase 1: Canary Testing (Week 7)
- Deploy to 10% of development teams
- Monitor cache hit rates and performance metrics
- Collect feedback on developer experience

#### Phase 2: Gradual Expansion (Week 8)
- Expand to 50% of teams based on Phase 1 success
- Enable background warming for high-usage projects
- Optimize cache strategies based on real usage patterns

#### Phase 3: Full Deployment (Week 9)
- Deploy to all teams with full feature set
- Enable advanced pattern learning and predictive warming
- Monitor enterprise-scale performance metrics

### Success Metrics
- **Cache Hit Rate**: >80% sustained over 1 week
- **Performance Improvement**: >75% faster for cached scenarios
- **Developer Satisfaction**: >4.5/5 rating for hot reload experience
- **Resource Usage**: <500MB cache size per project
- **Reliability**: <1% cache-related failures

---

## 💼 Business Impact

### Developer Productivity Gains
- **Immediate Feedback**: Near-instant pre-commit validation
- **Context Switching Reduction**: 75% fewer build waits
- **Flow State Preservation**: Uninterrupted development cycles
- **Confidence Increase**: Predictable, fast validation results

### Infrastructure Cost Savings
- **CI/CD Resource Reduction**: 50% fewer compute cycles
- **Build Server Optimization**: Better resource utilization
- **Network Traffic Reduction**: Cached results reduce remote calls
- **Storage Efficiency**: Content-addressable deduplication

### Competitive Advantage
- **Enterprise Differentiation**: Unique hot reload capability
- **Developer Experience**: Best-in-class pre-commit performance
- **Scalability**: Handles large monorepos with ease
- **Innovation Leadership**: Pioneering incremental execution in pre-commit space

---

## 🔮 Future Enhancements

### Planned Features (v0.4.0+)
1. **Distributed Caching**: Share cache across team members
2. **ML-Powered Prediction**: Advanced pattern learning with neural networks
3. **IDE Integration**: Real-time cache status in development environments
4. **Cloud Sync**: Centralized cache for distributed teams
5. **Analytics Dashboard**: Web-based cache performance monitoring

### Research Directions
- **Semantic Caching**: Use code semantics for more intelligent invalidation
- **Incremental Compilation**: Integration with language-specific incremental builds
- **Collaborative Warming**: Team-based predictive cache warming
- **Cross-Repository Caching**: Cache sharing between related projects

---

## 📚 Technical References

### Dependencies
```toml
[dependencies]
# Existing smart-hooks dependencies
# ... existing deps ...

# Hot reload specific dependencies
blake3 = "1.5"           # Fast content hashing
notify = "6.1"           # File system watching
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"          # Efficient cache serialization
lru = "0.12"             # LRU cache implementation
dashmap = "5.5"          # Concurrent HashMap
tracing = "0.1"          # Structured logging

[dev-dependencies]
tempfile = "3.8"         # Testing utilities
criterion = "0.5"        # Performance benchmarking
```

### Architecture Patterns
- **Cache-Aside Pattern**: Manual cache management with fallback
- **Write-Through Caching**: Immediate persistence of cache updates
- **Content-Addressable Storage**: Git-style content hashing
- **Event-Driven Architecture**: File system events drive cache invalidation
- **Background Processing**: Async warming and maintenance tasks

---

**Document Status**: 📋 **Ready for Implementation**
**Next Steps**: Begin Phase 1 implementation with cache storage foundation
**Review Schedule**: Weekly progress reviews, architecture adjustments as needed

---

*Hot Reload Architecture - Transforming Smart-Hooks into the fastest pre-commit system available* 🚀
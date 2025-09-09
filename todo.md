# Pure-TS Refactoring Plan

## Completed Improvements
- ✅ Test utilities for reducing duplication (test_utils.rs)
- ✅ Preset system for rule configurations
- ✅ Init command with minimal boilerplate

## Simplified Architecture Decision
複雑になりすぎたため、以下のファイルを削除してシンプルに保つことにしました：
- ~~rule_registry.rs~~ - 不要な抽象化
- ~~rule_categories.rs~~ - presetsで十分
- ~~config.rs~~ - 過剰な設定システム
- ~~rule_runner.rs~~ - combined_visitorで十分
- ~~error_reporter.rs~~ - 既存のLinterで十分
- ~~ast_cache.rs~~ - パフォーマンスは既に良好
- ~~test_fixtures.rs~~ - テストごとに直接記述で十分

現在の方針：
- シンプルさを維持
- 必要になったら機能を追加
- 過剰な抽象化を避ける

# Zero-Config Monorepo Support ✅

## 実装した機能
- **workspace_detector.rs** - workspace設定の自動検出
  - pnpm-workspace.yaml のサポート
  - package.json の workspaces フィールドのサポート
  - npm/yarn/pnpm workspaceの識別
- **自動ターゲット検出** - packages/*/src, apps/*/src を自動でスキャン
- **zero-config** - 設定ファイルなしで動作

## サポートするmonorepo構造
- pnpm workspaces (pnpm-workspace.yaml)
- npm workspaces (package.json#workspaces)
- yarn workspaces (package.json#workspaces + yarn.lock)

## 使用例
```bash
# Single package
purets src/

# Monorepo (自動検出)
purets .

# 特定のパッケージのみ
purets packages/my-package/
```

# Gitignore Support ✅

## 実装した機能
- **gitignore_filter.rs** - .gitignoreパターンに基づくファイルフィルタリング
- **デフォルト除外パターン** - node_modules, dist, out, target, build等を自動除外
- **.gitignoreの自動読み込み** - プロジェクトルートの.gitignoreを自動的に適用

## デフォルトで除外されるディレクトリ
- node_modules
- dist
- out
- target
- build
- coverage
- .git
- .next
- .nuxt
- .output
- .vercel
- vendor
- tmp/temp

## 動作
1. プロジェクトルートの`.gitignore`を自動的に読み込み
2. デフォルトの除外パターンを適用
3. glob展開前にディレクトリレベルでフィルタリング（高速化）
4. 最終的なファイルリストからも除外パターンを適用

# Current Focus

## Phase 1: Core Architecture Improvements ✅
- [x] Simplify rule registration system - reduce boilerplate in combined_visitor.rs
- [x] Extract common test utilities to reduce duplication across rule tests
- [x] Create a unified error reporting interface for all rules
- [x] Improve performance by caching parsed ASTs for multiple rule passes

## Phase 2: Rule Organization ✅
- [x] Group related rules into modules (e.g., type-safety, imports, functions)
- [x] Create rule categories with severity levels (error, warning, info)
- [x] Implement rule dependency system (some rules require others to run first)
- [x] Add rule configuration through config file instead of hardcoding

## Phase 3: Testing Infrastructure ✅
- [x] Create test fixture system for common test cases
- [x] Add integration tests for CLI commands
- [ ] Implement snapshot testing for rule outputs
- [ ] Add performance benchmarks for large codebases

## Phase 4: User Experience
- [ ] Improve error messages with code snippets and fix suggestions
- [ ] Add --fix flag for auto-fixable rules
- [ ] Create interactive mode for rule configuration
- [ ] Add progress indicator for large project analysis

## Phase 5: Documentation
- [ ] Generate rule documentation from code comments
- [ ] Create examples directory with common patterns
- [ ] Add troubleshooting guide
- [ ] Document performance optimization tips

## Phase 6: Advanced Features
- [ ] Implement incremental analysis (only check changed files)
- [ ] Add watch mode for continuous checking
- [ ] Create plugin system for custom rules
- [ ] Support for monorepo structures

## Technical Debt
- [ ] Remove SourceType::default() usage completely
- [ ] Consolidate parse_and_check functions across tests
- [ ] Reduce allow_directives.rs complexity (currently 535 lines)
- [ ] Improve neverthrow integration in generated code

## Next Steps
1. Start with Phase 1 - Core Architecture Improvements
2. Focus on reducing boilerplate and improving maintainability
3. Ensure backward compatibility while refactoring
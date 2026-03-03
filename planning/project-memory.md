# Project Memory

## Iteration 2 — Style Serialization in .sui Format

### Requirements Review (2026-03-02, Revision 1)

**Verdict: NEEDS REVISION**

#### Critical Finding: font.sz is Not a Valid ironcalc Style Path

The spec lists `font.sz` as a supported style key in the grammar (`style_key` production) and in REQ-001/REQ-006. However, the ironcalc `update_range_style` API does **not** accept `"font.sz"` as a path string. The only font-size-related path is `"font.size_delta"`, which applies a relative increment to the current size (not an absolute setter).

Verified in: `ironcalc_base-0.7.1/src/user_model/common.rs`, lines 116-141.
The `Style.font.sz` field (i32) exists internally, but there is no absolute-value setter path exposed via `update_range_style`.

**Implication**: Either `font.sz` must be dropped from the spec scope, or the spec must document a workaround (e.g., compute delta from default, use direct model manipulation). This is a blocking issue because a round-trip test for `font.sz` would require calling `update_range_style` with a path that does not exist.

#### Grammar style_val is Under-specified for alignment values

The `style_val` production only allows `'true' | 'false' | quoted_string | hex_color`. Alignment values like `"center"`, `"left"`, `"justify"` are not quoted strings in the example — they would need to either be bare tokens or quoted. The spec is ambiguous on this point.

#### No Schema File Present

The `schemas/requirements.schema.yaml` referenced in the critic instructions does not exist in this repository. Schema validation was performed manually against the checklist.

### ironcalc Style API Reference (for downstream agents)

Valid `update_range_style` path strings (ironcalc_base 0.7.1):
- `"font.b"` — bool (true/false)
- `"font.i"` — bool
- `"font.u"` — bool  
- `"font.strike"` — bool
- `"font.color"` — hex color `#RRGGBB` (7 chars, exactly)
- `"font.size_delta"` — signed integer (relative increment, NOT absolute)
- `"fill.bg_color"` — hex color
- `"fill.fg_color"` — hex color
- `"num_fmt"` — format string (e.g., `"$#,##0.0000"`, `"general"`)
- `"alignment"` — empty string only (clears alignment)
- `"alignment.horizontal"` — one of: center, centerContinuous, distributed, fill, general, justify, left, right
- `"alignment.vertical"` — one of: bottom, center, distributed, justify, top
- `"alignment.wrap_text"` — bool

**There is no `"font.sz"` path.** The font size field `Style.font.sz` (default 13) can only be changed via `"font.size_delta"` (relative).

Valid hex color format: exactly `#RRGGBB` (7 characters, uppercase or lowercase hex digits, no alpha channel).

### Canonical Ordering Note

The existing `serialize_sui` implementation orders col_width declarations before cell_decl lines. Style declarations should follow the same row-major ordering established for cells.

### Requirements Review (2026-03-02, Revision 2)

**Verdict: NEEDS REVISION**

#### Blocking Issues from Revision 1: Status

1. font.sz removal — RESOLVED. Removed from style_key grammar; added to out_of_scope with full explanation; added to assumptions section.
2. style_val alignment ambiguity — RESOLVED. Separate align_h_val and align_v_val productions added with explicit bare token enumerations matching the ironcalc API exactly.

#### New Blocking Issue Found in Revision 2

**REQ-002 acceptance criterion still says "twelve supported style properties"** (line 97 of requirements.yaml). Every other occurrence in the document correctly says "eleven" (REQ-001 line 80, REQ-006 line 150, done_criteria line 218). The property count is eleven after removing font.sz. This is an internal inconsistency introduced by an incomplete find-replace during the revision.

Fix required: change "twelve" to "eleven" on the REQ-002 acceptance criterion line.

#### Observations (Non-Blocking)

- Grammar overlap between align_h_val and align_v_val (both include center, distributed, justify) is not a real ambiguity problem in practice — a parser disambiguates by the preceding style_key. The grammar is documentary, not a formal LL/LR spec. Acceptable.
- The bare "alignment" path (empty string, clears alignment) is not in scope; this is correct — it is a clear operation, not a set-value operation.
- None-fill-color sentinel and lowercase boolean requirements are now clearly documented in assumptions. Downstream implementors have all needed information.

### Architecture Review (2026-03-02, Revision 1)

**Verdict: APPROVED WITH RECOMMENDATIONS**

#### Files Reviewed

- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/architecture_index.yaml`
- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/architecture_components.yaml`
- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/architecture_traceability.yaml`
- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/architecture_adr_002.yaml`
- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/architecture_dependencies.yaml`
- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/architecture_data_model.yaml`

#### Blocking Issues: NONE

All prior requirements-review blocking issues (font.sz removal, style_val alignment ambiguity, "twelve" vs "eleven" count) are resolved in the submitted artifacts. No new blocking issues found.

#### Recommended Issues (non-blocking)

1. **COMP-001 requirements_addressed contains stale iteration-1 IDs** — The `architecture_components.yaml` COMP-001 `requirements_addressed` list includes REQ-009, REQ-011, REQ-013, REQ-015, which are iteration-1 requirements absent from the iteration-2 requirements file. While technically harmless, this creates a false impression that those IDs are being addressed by the current spec and could cause confusion for a developer or future critic reading components in isolation. Recommend moving those IDs to a comment or a separate `legacy_requirements_addressed` field.

2. **ADR-002 requirements_addressed is missing REQ-006 and REQ-008** — The ADR lists iter2/REQ-001 through iter2/REQ-005 and iter2/REQ-007 but omits iter2/REQ-006 (test coverage mandate) and iter2/REQ-008 (SUI_FORMAT.md documentation). Both are clearly addressed by COMP-001's design decisions in the ADR (ADR-002-D3 for lenient parsing enabling REQ-006 test predictability; ADR-002-D6 for the value encoding that must be documented in REQ-008). Recommend adding both to the ADR's requirements_addressed list for traceability completeness.

3. **Linter ruleset is "default" rather than strict/pedantic** — The architecture_index.yaml specifies `ruleset: "default"` for both rustc and clippy. The checklist requires linter rulesets to start strict/pedantic with relaxations documented in an ADR. The zero-warnings policy is enforced, but no `#![deny(warnings)]` or `#![warn(clippy::all, clippy::pedantic)]` configuration is specified, and no ADR documents the chosen relaxations. For a narrow file-format change this is a low risk in practice, but it should be addressed in the architecture documentation for completeness.

#### Suggestions (truly optional)

- The integration_test_boundaries entry on COMP-001 points at COMP-002 (UI reads styles from Book) which is correct for the overall component contract. An additional boundary note covering the sui.rs internal contract (serialize then parse produces identical style state) would make the test strategy self-documenting and help the test critic stage.

- The ADR alternatives_considered section is thorough and well-reasoned. A brief note acknowledging the streaming-parser advantage of the line-oriented approach (versus a block-oriented [styles] section) exists in "Add a separate [styles] section block" but could call out the memory implication explicitly (streaming parse, no need to buffer).

#### Summary

The architecture is correct, implementable, and well-reasoned. The scope is precisely bounded to COMP-001 (src/book/sui.rs). All eight iteration-2 requirements are mapped with notes in the traceability file. ADR-002 documents all six required decisions with alternatives considered and consequences stated. The component boundary is correct: no UI components (COMP-002 through COMP-008) are listed as affected. The constraint of no new dependencies is upheld — no new packages appear in architecture_dependencies.yaml. The grammar extension is backward-compatible (old parsers emit warnings, do not abort) and forward-compatible (new parsers handle style-less files identically to before).

### Implementation Review — Phase 2 (2026-03-02, Revision 1)

**Verdict: APPROVED**

#### Build and Test Results

- `cargo build`: PASSED, zero warnings
- `cargo test`: PASSED, 154 tests, zero warnings, zero failures
  - 19 iter-2 style tests: all green
  - 19 iter-1 sui tests: all green (no regressions)
  - 116 pre-existing ui/book tests: all green

#### Checklist Results

- **Zero compiler warnings**: CONFIRMED
- **All tests pass**: CONFIRMED (154/154)
- **No `#[allow(...)]` suppressions**: CONFIRMED — all three iter-1 dead_code suppressions removed
- **No unsafe code**: CONFIRMED
- **Helper functions are private (not pub)**: CONFIRMED — `is_default_style`, `serialize_style_props`, `parse_style_decl`, `apply_style_props` are all private fn
- **Canonical ordering in serialize_sui**: CONFIRMED — col_width lines (lines 184-191), then style_decl lines (lines 193-215), then cell_decl lines (lines 217-240)
- **parse_sui handles style_decl before cell_decl**: CONFIRMED — `else if let Some(...) = parse_style_decl(...)` (line 130) appears before `else if let Some(...) = parse_cell_decl(...)` (line 132)
- **Requirements REQ-001 through REQ-007**: ALL IMPLEMENTED. REQ-008 (docs/SUI_FORMAT.md) deferred to documentation phase per implementation plan
- **No test files modified**: CONFIRMED — test section content matches Phase 1 approved state; 19 new style tests present; no tests deleted

#### Non-Blocking Finding: EBNF Grammar Comment Not Updated

- File: `/Users/zaphar/projects/personal/sheetui/src/book/sui.rs`, lines 23-41
- The module-level doc comment EBNF grammar still shows the old `line` production without `style_decl`. The grammar does not include `style_decl`, though the implementation supports it.
- This is a documentation inconsistency only. No functional impact. The inline implementation comments (line 193: "canonical: after col_widths, before cell_decls") and the `parse_style_decl` doc comment are correct.
- Recommend fixing in documentation phase alongside `docs/SUI_FORMAT.md` creation (REQ-008).

#### Non-Blocking Finding: Display trait used for alignment enum serialization

- File: `/Users/zaphar/projects/personal/sheetui/src/book/sui.rs`, lines 440, 443
- `serialize_style_props` uses `format!("{}", alignment.horizontal)` and `format!("{}", alignment.vertical)` relying on Display trait, rather than explicit match arms.
- Implementation plan notes explicit match arms are "preferred" but this is not a hard requirement. The Display impl produces correct tokens ("center", "top", etc.) as verified by passing tests.
- The risk is that a future ironcalc upgrade could change Display output format. This is acceptable given the version is pinned in Cargo.lock.

#### Patterns Confirmed Working

- Double-guard in `serialize_sui` (`is_default_style` + `!props.is_empty()`) is redundant but harmless defensive coding.
- Empty-cell styling (styled-but-no-value cells) correctly appear in sheet_data via ironcalc's EmptyCell mechanism; serialize/parse round-trip works correctly.
- `parse_style_decl` correctly handles the edge case of "style A1" with no properties (returns empty props vec), avoiding a malformed-line warning.
- The `apply_style_props` unknown-key warning correctly emits one warning per unknown key (not one per style line), which is the right granularity.

#### Summary

Phase 2 is complete and correct. All 7 in-scope requirements (REQ-001 through REQ-007) are implemented and verified by passing tests. The implementation is clean, secure (no injection vectors, no unsafe code), and maintains the zero-warnings policy throughout.

### Documentation Review (2026-03-03, Revision 1)

**Verdict: APPROVED**

#### Documents Reviewed

- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/rigor-workflow/documentation/documentation_manifest.yaml`
- `/Users/zaphar/projects/personal/sheetui/docs/SUI_FORMAT.md`
- `/Users/zaphar/projects/personal/sheetui/docs/file-formats.md`
- `/Users/zaphar/projects/personal/sheetui/src/book/sui.rs` (lines 1-75, module doc comment)
- `/Users/zaphar/projects/personal/sheetui/.claude/rigor-artifacts/rigor-workflow/requirements/requirements.yaml` (REQ-008 acceptance criteria)
- `/Users/zaphar/projects/personal/sheetui/docs/index.md`

#### REQ-008 Acceptance Criteria: ALL PASSED

1. `docs/SUI_FORMAT.md` exists and describes the complete `.sui` file format — PASS
2. Full EBNF grammar including `style_decl` production present — PASS
3. All 11 style keys listed with value type and example — PASS
4. `alignment.horizontal` all 8 values enumerated — PASS
5. `alignment.vertical` all 5 values enumerated — PASS
6. Backward compatibility note (ParseWarning, no abort) present — PASS
7. Fully-annotated example with cell values, col widths, and style declarations — PASS

#### Non-Blocking Implementation Finding Resolved

The implementation review (2026-03-02) noted as non-blocking that the EBNF grammar in `src/book/sui.rs` had not been updated to include `style_decl`. This was resolved in the documentation phase: lines 25-53 of `sui.rs` now contain the full updated grammar matching `SUI_FORMAT.md` exactly.

#### Recommendations (non-blocking)

1. Module doc example in `src/book/sui.rs` (lines 64-75) does not show a `style_decl` line between col and cell declarations. The grammar is correct; the example is incomplete.
2. Manifest `summary.documents_created: 1` is technically correct (1 new file) but `file-formats.md` was also updated; a `documents_updated` field would be clearer.
3. Manifest getting_started description says "updated with link to SUI_FORMAT.md" but the actual link goes to `file-formats.md` (which then links to `SUI_FORMAT.md`). Minor imprecision.

#### Patterns/Lessons

- When a format-extension feature adds new syntax, the module-level doc comment grammar should be updated in the implementation phase (not deferred to docs). This avoided a blocking finding but required an extra pass.
- Cross-referencing strategy (index -> file-formats -> SUI_FORMAT) is navigable and appropriate for a layered format spec. No single-document requirement needed.

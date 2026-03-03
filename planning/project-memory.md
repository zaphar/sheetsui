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

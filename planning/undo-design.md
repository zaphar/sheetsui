# Design: Undo and Redo

Status: accepted 2026-10-28 — decisions D1–D7. Issue #45.

## User

A person who edits a sheet and makes a mistake: a wrong edit, a paste over
data, a sort by the wrong column, a fill that went too far. They want to go
back one action or more, and maybe forward again.

## Requirements (issue #45)

- Every action that changes the book can be undone.
- The history holds at least 1024 actions.

## Flow

1. In Navigate mode, the user types `u`. The last action is undone. The
   cursor goes to the cell where that action was done.
2. `r` redoes the last undone action. The cursor goes to the same place.
3. A count repeats: `3u` undoes three actions, `3r` redoes three.
4. `:undo` and `:redo` do the same as `u` and `r`.
5. A new action after an undo clears the redo history.

### Keys

| Key or command | Mode | What it does |
|----------------|------|--------------|
| `u` | Navigate | Undo one action. |
| `r` | Navigate | Redo one action. |
| `N u`, `N r` | Navigate | Undo or redo N actions. |
| `:undo`, `:redo` | Command | Undo or redo one action. |
| `Ctrl-r` | Navigate, CellEdit | No change: enters Range Select. |

`u` and `r` are not bound in any other mode. In CellEdit they type letters.

## What is one action

One input event that changes the book is one action: one key press in
Navigate or Range Select mode, one command, or one cell edit committed with
Enter. So these each undo in one step:

| Action | ironcalc steps (example) |
|--------|--------------------------|
| Commit a cell edit | 1 |
| `p` paste of a 3 × 4 range | 12 |
| `5 Ctrl-l` (widen the column 5 times) | 5 |
| `:insert-rows 5` | 5 |
| `:color-rows 3 red` | 3 |
| `:sort` of 10,000 × 12 cells | 120,002 |
| `d`/`D` on a range | 1 |

Each of these can be undone:

- cell edits
- clear contents and clear all
- paste
- fill (extend)
- insert rows and columns
- bold, italic, and colors
- column width
- new sheet and rename sheet
- sort

These are not actions and are not in the history:

- moving the cursor and making a selection
- switching sheets
- saving
- loading a file (`:e`, or on start)

`o` and `O` are two actions: the row insert, then the cell edit when you
commit it. `u` after `o` + typing + Enter undoes the edit; a second `u`
removes the row. (In vim, `o` and the typed text are one change. Here the
edit is a separate mode, so they stay apart.)

## Cases

| Case | Behavior |
|------|----------|
| `u` with nothing to undo | Nothing happens. No message. |
| `r` with nothing to redo | Nothing happens. No message. |
| `5u` with only 2 actions | Undoes the 2 actions. |
| A key or command that changes nothing (a sort that is already in order) | Not an action. `u` undoes the action before it. |
| An action that fails part way (an error after some cells changed) | The part that changed is one action, so `u` can restore it. |
| Undo of `:new-sheet` while the cursor is on the new sheet | The sheet goes away; the cursor goes to the sheet and cell where `:new-sheet` was typed. |
| The recorded cell's sheet no longer exists | The cursor goes to the last sheet. |
| Open another file (`:e`) | The new book has an empty history. |
| Restart sheetui | The history is not saved. It starts empty. |
| Large sort (10,000 × 12) | Undo 264 ms, redo 410 ms (measured, release build). |

## Saved state (D4)

The book tracks the history position of the last save.

- Undo or redo back to that position: the book is not changed. `q` quits
  with no prompt.
- Any other position: the book is changed.
- A new action after undoing past the save point: the save point can no
  longer be reached, so the book stays changed until the next save.

**Fix included:** today `Book::move_to` sets `dirty = true`, so moving the
cursor makes `q` prompt to save. The cursor position is not part of the
file, so `move_to` stops setting `dirty`. After this change, `q` prompts
only when the book has unsaved changes.

## Technical approach (D5)

ironcalc's `UserModel` already records an undo step (a diff list) for every
change, with `undo`, `redo`, `can_undo`, and `can_redo`. Its stacks have no
size limit. It has no public way to group steps, so one sheetui action can be
thousands of ironcalc steps.

sheetui keeps its own history of actions on top of ironcalc's:

```rust
struct Action {
    /// How many ironcalc undo steps this action made.
    steps: usize,
    /// Where the cursor was when the action started.
    location: Address,
}

struct UndoHistory {
    done: Vec<Action>,
    undone: Vec<Action>,
    /// `done.len()` at the last save; `None` if it cannot be reached.
    save_point: Option<usize>,
    /// The action being recorded now, if any.
    open: Option<Action>,
}
```

### Counting steps

Every ironcalc call that can change the book goes through one `Book` helper.
The helper empties `UserModel::flush_send_queue` before the call and checks it
after. ironcalc puts every step it records in that queue, so a queue that is
not empty means the call made a step. This is more reliable than a table of
"call X makes one step": some ironcalc calls make no step when they change
nothing (`on_paste_styles` with no change, `set_name` to the same name).

The ironcalc calls sheetui uses make at most one step each. Calls that make
two (`range_clear_formatting`, borders, clipboard, autofill) are not used.
A test for each action checks that `u` restores the book exactly. If a
future ironcalc version records steps differently, these tests fail.

Emptying the queue also fixes a leak. sheetui never emptied it, so it held a
copy of every change since the file was opened.

### Grouping

- `Workspace` opens an action before it handles each input event and closes
  it after. An action with 0 steps is dropped.
- A change made with no open action is its own action, so no ironcalc step is
  ever left out of the count.
- Loading a book clears the history at the end. Its ironcalc steps stay
  below the bottom of the sheetui history, so they can never be undone.

### Undo and redo

- Undo: pause evaluation, call `UserModel::undo` `steps` times, resume, and
  evaluate one time. Then move the cursor to the action's location and update
  `dirty` from the save point. Empty the send queue (undo puts its own
  entries there).
- Redo: the same with `UserModel::redo`.
- A new action clears `undone`. ironcalc clears its own redo stack on every
  new step, so the two stay in step.

Evaluation must be paused. Unpaused, redo of a 1,000-row sort took 11 s
(it evaluates after each step). Paused, 10,000 rows take 410 ms.

### History size

sheetui's history has no limit, so it holds more than 1024 actions. A limit
would not save memory: ironcalc keeps every step on its own stack, and it has
no public way to drop old steps.

### Alternatives considered

- **Add a batch API to ironcalc** (begin/end group, one diff list). Cleaner,
  but sheetui would need a fork as a git dependency until a release has it.
  Possible later; the sheetui history would then count 1 step per action.
- **Save a snapshot of the book (`to_bytes`) per action.** Rejected: a
  10,000-row book is megabytes, times 1024 actions.
- **Record sheetui's own cell diffs.** Rejected: it repeats what ironcalc
  already records, for every kind of change.
- **A fixed table of steps per sheetui method.** Rejected: wrong when an
  ironcalc call makes no step because nothing changed.

## Decisions

- **D1 — Keys: `u` undo, `r` redo, in Navigate mode.** Counts work. No
  capital-letter variants. `:undo` and `:redo` as commands.
- **D2 — `Ctrl-r` does not change.** It still enters Range Select in
  Navigate and CellEdit. Rejected: vim's `Ctrl-r` for redo, because Range
  Select would lose its key (a regression), and `V` is kept free for a
  future whole-row select.
- **D3 — Cursor goes to the change.** After undo or redo, the cursor goes to
  the sheet and cell where the action was done.
- **D4 — Save point is tracked.** Undo back to the saved state makes the book
  not changed. `move_to` no longer sets `dirty`.
- **D5 — Count ironcalc steps in sheetui.** No ironcalc change. See above.
- **D6 — One input event is one action.** A count (`5 Ctrl-l`), a multi-cell
  paste, or a multi-row command undoes in one step. `o`/`O` and the edit after
  them are two actions.
- **D7 — History is not saved** and has no size limit.

## Docs to update

- `docs/navigation.md` — `u`, `r`.
- `docs/command.md` — `:undo`, `:redo`; remove the "no undo" note in
  Sorting.
- `docs/visual.md` — none (no new keys in Range Select).

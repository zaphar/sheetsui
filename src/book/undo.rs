//! Undo and redo.
//!
//! ironcalc records an undo step for every change, but one sheetui action
//! can be many steps: a paste writes each cell, and a sort rewrites every
//! cell of the sorted rows. ironcalc has no public way to group steps, so
//! sheetui keeps its own history of actions. Each action knows how many
//! ironcalc steps it made, and undo replays that many.
//!
//! ironcalc also puts every step it records in its send queue. Every
//! ironcalc call that can change the book goes through [`Book::change`],
//! which empties the queue after the call and counts a step if the queue held
//! one. Every ironcalc call that sheetui uses records at most one step.
use anyhow::{anyhow, Result};
use ironcalc::base::UserModel;

use super::Book;
use crate::ui::Address;

#[derive(Debug, Clone)]
struct Action {
    /// How many ironcalc undo steps this action made.
    steps: usize,
    /// Where the cursor was when the action started.
    location: Address,
}

#[derive(Debug)]
pub(crate) struct UndoHistory {
    done: Vec<Action>,
    undone: Vec<Action>,
    /// `done.len()` at the last save, or `None` if undo and redo can no
    /// longer reach it.
    save_point: Option<usize>,
    /// The action being recorded now, if any.
    open: Option<Action>,
    /// How many [`Book::begin_action`] calls have no [`Book::end_action`]
    /// yet. The action ends when this goes back to 0.
    depth: usize,
    /// What `flush_send_queue` returns when the queue is empty.
    empty_queue: Vec<u8>,
}

impl UndoHistory {
    /// A history for `model`. Empties its send queue.
    pub(crate) fn new(model: &mut UserModel) -> Self {
        model.flush_send_queue();
        Self {
            done: Vec::new(),
            undone: Vec::new(),
            save_point: Some(0),
            open: None,
            depth: 0,
            empty_queue: model.flush_send_queue(),
        }
    }

    fn push(&mut self, action: Action) {
        self.undone.clear();
        if self.save_point.is_some_and(|p| p > self.done.len()) {
            self.save_point = None;
        }
        self.done.push(action);
    }

    fn is_at_save_point(&self) -> bool {
        self.save_point == Some(self.done.len())
    }
}

impl Book {
    /// Run an ironcalc call that can change the book, and count the undo
    /// step it made, if any.
    pub(super) fn change<T>(
        &mut self,
        call: impl FnOnce(&mut UserModel<'static>) -> Result<T, String>,
    ) -> Result<T, String> {
        let result = call(&mut self.model);
        self.count_step();
        result
    }

    fn count_step(&mut self) {
        if self.model.flush_send_queue() == self.undo.empty_queue {
            return;
        }
        match &mut self.undo.open {
            Some(action) => action.steps += 1,
            None => {
                let location = self.location.clone();
                self.undo.push(Action { steps: 1, location });
            }
        }
    }

    /// Start an action. Changes until the matching [`Book::end_action`] undo
    /// as one. Actions nest: an action started inside another is part of it.
    pub fn begin_action(&mut self) {
        if self.undo.depth == 0 {
            self.undo.open = Some(Action {
                steps: 0,
                location: self.location.clone(),
            });
        }
        self.undo.depth += 1;
    }

    /// End the action that [`Book::begin_action`] started. An action that
    /// changed nothing is not kept.
    pub fn end_action(&mut self) {
        if self.undo.depth == 0 {
            return;
        }
        self.undo.depth -= 1;
        if self.undo.depth > 0 {
            return;
        }
        if let Some(action) = self.undo.open.take() {
            if action.steps > 0 {
                self.undo.push(action);
            }
        }
    }

    /// Run `f` as one action.
    pub(super) fn as_action<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.begin_action();
        let result = f(self);
        self.end_action();
        result
    }

    /// Undo the last action. Returns false if there is nothing to undo.
    pub fn undo(&mut self) -> Result<bool> {
        let Some(action) = self.undo.done.pop() else {
            return Ok(false);
        };
        self.replay(action.steps, UserModel::undo)?;
        self.go_to_action(&action)?;
        self.undo.undone.push(action);
        self.dirty = !self.undo.is_at_save_point();
        Ok(true)
    }

    /// Redo the last undone action. Returns false if there is nothing to
    /// redo.
    pub fn redo(&mut self) -> Result<bool> {
        let Some(action) = self.undo.undone.pop() else {
            return Ok(false);
        };
        self.replay(action.steps, UserModel::redo)?;
        self.go_to_action(&action)?;
        self.undo.done.push(action);
        self.dirty = !self.undo.is_at_save_point();
        Ok(true)
    }

    /// Record that the book was saved now.
    pub(super) fn mark_saved(&mut self) {
        self.undo.save_point = Some(self.undo.done.len());
        self.dirty = false;
    }

    /// Forget all actions. A book that was just loaded has nothing to undo.
    pub(super) fn clear_history(&mut self) {
        self.undo = UndoHistory::new(&mut self.model);
    }

    fn replay(
        &mut self,
        steps: usize,
        step: fn(&mut UserModel<'static>) -> Result<(), String>,
    ) -> Result<()> {
        // Evaluating after each step makes a large undo take minutes.
        self.model.pause_evaluation();
        let result = (0..steps).try_for_each(|_| step(&mut self.model));
        self.model.resume_evaluation();
        // Undo and redo put their own entries in the send queue.
        self.model.flush_send_queue();
        result.map_err(|e| anyhow!(e))?;
        self.evaluate();
        Ok(())
    }

    fn go_to_action(&mut self, action: &Action) -> Result<()> {
        let last = self
            .model
            .get_model()
            .workbook
            .worksheets
            .len()
            .saturating_sub(1) as u32;
        let sheet = action.location.sheet.min(last);
        self.model
            .set_selected_sheet(sheet)
            .map_err(|e| anyhow!(e))?;
        self.location = Address {
            sheet,
            ..action.location.clone()
        };
        Ok(())
    }
}

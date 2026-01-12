// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::FluxionTask;

/// Task guard that cancels the partition routing task when dropped.
///
/// This guard is shared between both partition streams via Arc.
/// When the last stream is dropped, the task is automatically cancelled.
#[derive(Debug)]
pub(super) struct TaskGuard {
    pub(super) task: FluxionTask,
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.task.cancel();
    }
}

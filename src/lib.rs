#![doc=include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::marker::PhantomData;
use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::{ScheduleLabel, InternedScheduleLabel}};

/// Adds progress tracking for `T` (as a resource).
pub struct ResourceProgressTrackingPlugin<T: ?Sized> {
    /// The schedule in which the progress value is checked.
    pub check_schedule: InternedScheduleLabel,

    /// The schedule in which the progress value is checked.
    /// This should be the same as, or before, `check_schedule`.
    pub reset_schedule: InternedScheduleLabel,

    _p1: PhantomData<T>,
}

impl<T: ?Sized> Default for ResourceProgressTrackingPlugin<T> {
    fn default() -> Self {
        Self {
            check_schedule: PostUpdate.intern(),
            reset_schedule: Last.intern(),
            _p1: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Plugin for ResourceProgressTrackingPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(self.check_schedule, resource_progress_check_system::<T>
            .in_set(ProgressSystems::Check));

        app.add_systems(self.reset_schedule, resource_progress_reset_system::<T>
            .in_set(ProgressSystems::Reset)
            .after(ProgressSystems::Check));
    }
}

fn resource_progress_check_system<T: ?Sized + Send + Sync + 'static>(
    mut commands: Commands,
    resource: Option<Res<Progress<T>>>,
) {
    let resource = match resource {
        Some(v) => v,
        None => return,
    };

    if !resource.done() { return }
    commands.trigger(Done::<T> {
        work: resource.total,
        _p1: PhantomData,
    });
}

fn resource_progress_reset_system<T: ?Sized + Send + Sync + 'static>(
    resource: Option<ResMut<Progress<T>>>,
) {
    if let Some(mut resource) = resource {
        resource.done = 0;
        resource.total = 0;
    }
}

/// Adds progress tracking for `T` (as a component).
pub struct EntityProgressTrackingPlugin<T: ?Sized> {
    /// The schedule in which the progress value is checked.
    pub check_schedule: InternedScheduleLabel,

    /// The schedule in which the progress value is checked.
    /// This should be the same as, or before, `check_schedule`.
    pub reset_schedule: InternedScheduleLabel,

    _p1: PhantomData<T>,
}

impl<T: ?Sized> Default for EntityProgressTrackingPlugin<T> {
    fn default() -> Self {
        Self {
            check_schedule: PostUpdate.intern(),
            reset_schedule: Last.intern(),
            _p1: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Plugin for EntityProgressTrackingPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(self.check_schedule, entity_progress_check_system::<T>
            .in_set(ProgressSystems::Check));

        app.add_systems(self.reset_schedule, entity_progress_reset_system::<T>
            .in_set(ProgressSystems::Reset)
            .after(ProgressSystems::Check));
    }
}

fn entity_progress_check_system<T: ?Sized + Send + Sync + 'static>(
    mut commands: Commands,
    query: Query<(Entity, &Progress<T>)>,
) {
    for (entity, tracker) in &query {
        if !tracker.done() { continue }
        commands.trigger_targets(Done::<T> {
            work: tracker.total,
            _p1: PhantomData,
        }, [entity]);
    }
}

fn entity_progress_reset_system<T: ?Sized + Send + Sync + 'static>(
    mut query: Query<&mut Progress<T>>,
) {
    for mut tracker in &mut query {
        tracker.done = 0;
        tracker.total = 0;
    }
}

/// Systems involved in progress tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum ProgressSystems {
    /// System(s) that check for completed trackers.
    /// All progress should be recorded before this point.
    Check,

    /// Progress trackers are reset in preparation for the next tick.
    /// Progress should not be read after this point.
    Reset,
}

/// Progress state.
/// 
/// Can be inserted as a [`Resource`] to track global progress,
/// or as a [`Component`] to track progress for a single entity.
#[derive(Component, Resource)]
pub struct Progress<T: ?Sized> {
    done: u64,
    total: u64,
    _p1: PhantomData<T>,
}

impl<T: ?Sized> Progress<T> {
    /// Creates a new [`Progress`] tracker.
    pub fn new() -> Self {
        Self {
            done: 0,
            total: 0,
            _p1: PhantomData,
        }
    }
}

impl<T: ?Sized> Default for Progress<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Progress<T> {
    /// Records progress, including its total work and done work.
    pub fn track(&mut self, done: u32, total: u32) {
        self.done += done as u64;
        self.total += total as u64;
    }

    /// Returns the work that has been completed and the units of work 
    pub fn work(&self) -> (u64, u64) {
        (self.done, self.total)
    }

    /// Returns the progress as a fraction, from `0.0` (no work done) to `1.0` (all work done).
    pub fn fract(&self) -> f32 {
        let (done, total) = self.work();
        return done as f32 / total as f32;
    }

    fn done(&self) -> bool {
        let (done, total) = self.work();
        if total == 0 { return false }
        return done >= total;
    }
}

/// An observer event raised when a progress tracker completes.
#[derive(Event)]
pub struct Done<T: ?Sized> {
    work: u64,
    _p1: PhantomData<T>,
}

impl<T: ?Sized> Done<T> {
    /// Returns the amount of work done.
    #[inline]
    pub fn work(&self) -> u64 {
        self.work
    }
}
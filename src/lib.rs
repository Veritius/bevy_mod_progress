#![doc=include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{marker::PhantomData, sync::atomic::{AtomicU64, Ordering as AtomicOrdering}};
use bevy::{ecs::schedule::{ScheduleLabel, SystemConfigs}, prelude::*, utils::intern::Interned};

type Schedule = Interned<dyn ScheduleLabel>;

/// Types that can be used in the progress tracker plugin as a distinguishing value.
/// Automatically implemented for almost all types, so no need to worry about this.
pub trait ProgressType: Send + Sync + 'static {}
impl<T> ProgressType for T where T: Send + Sync + 'static {}

/// A simple progress tracker plugin.
/// Runs the [`Done`] schedule when finished.
pub struct ProgressTrackerPlugin<T: ProgressType> {
    /// The schedule where progress is checked to see if we need to finish.
    /// Set to [`Last`] by default.
    pub check_schedule: Schedule,

    /// Removes the [`OverallProgress`] resource when finished.
    pub remove_on_done: bool,

    #[doc(hidden)]
    pub phantom: PhantomData<T>,
}

impl<T: ProgressType> Default for ProgressTrackerPlugin<T> {
    fn default() -> Self {
        Self {
            check_schedule: Last.intern(),
            remove_on_done: true,
            phantom: PhantomData,
        }
    }
}

impl<T: ProgressType> Plugin for ProgressTrackerPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_schedule(Done::<T>::new());
        app.insert_resource(PluginConfig { remove_on_done: self.remove_on_done, phantom: PhantomData::<T> });
        app.insert_resource(OverallProgress(OverallProgressInner::default(), PhantomData::<T>));
        app.add_systems(self.check_schedule, schedule_check_system::<T>);
    }
}

#[derive(Resource)]
struct PluginConfig<T: ProgressType> {
    remove_on_done: bool,
    phantom: PhantomData<T>,
}

fn schedule_check_system<T: ProgressType>(
    world: &mut World,
) {
    let res = match world.get_resource::<OverallProgress<T>>() {
        Some(res) => res,
        None => { return },
    };

    if res.done() < res.required() { return }
    world.run_schedule(Done::<T>::new());

    if world.resource::<PluginConfig<T>>().remove_on_done {
        world.remove_resource::<OverallProgress<T>>();
    }
}

/// System set for systems that track progress.
#[derive(SystemSet)]
pub struct ProgressTrackingSet<T: ProgressType>(PhantomData<T>);

impl<T: ProgressType> ProgressTrackingSet<T> {
    /// An instance of the progress tracking set.
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ProgressType> std::fmt::Debug for ProgressTrackingSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ProgressTrackingSet<{}>", std::any::type_name::<T>()))
    }
}

impl<T: ProgressType> Clone for ProgressTrackingSet<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T: ProgressType> PartialEq for ProgressTrackingSet<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ProgressType> Eq for ProgressTrackingSet<T> {}

impl<T: ProgressType> std::hash::Hash for ProgressTrackingSet<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Schedule run when the progress tracker corresponding to `T` finishes.
#[derive(ScheduleLabel)]
pub struct Done<T: ProgressType>(PhantomData<T>);

impl<T: ProgressType> Done<T> {
    /// Makes a new instance of the schedule label.
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ProgressType> std::fmt::Debug for Done<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Done<{}>", std::any::type_name::<T>()))
    }
}

impl<T: ProgressType> Clone for Done<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T: ProgressType> PartialEq for Done<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ProgressType> Eq for Done<T> {}

impl<T: ProgressType> std::hash::Hash for Done<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Progress reported by a system.
#[derive(Debug, Clone, Default)]
pub struct Progress {
    /// The amount of work that has been done.
    pub done: u32,
    /// The amount of work that has to be done to progress.
    pub required: u32,
}

impl Progress {
    /// Records progress.
    #[inline]
    pub fn apply<T: ProgressType>(&self, overall: &OverallProgress<T>) {
        overall.apply(self);
    }
}

/// Overall recorded progress.
/// 
/// This value can change partway through a system, even if accessed through a `Res`.
#[derive(Resource)]
pub struct OverallProgress<T: ProgressType>(OverallProgressInner, PhantomData<T>);

impl<T: ProgressType> OverallProgress<T> {
    /// Creates a new progress tracker.
    /// Does nothing unless the plugin is also added.
    #[inline]
    pub fn new() -> Self {
        Self(OverallProgressInner::default(), PhantomData)
    }

    /// Records progress.
    #[inline]
    pub fn apply(&self, progress: &Progress) {
        self.0.apply(progress)
    }

    /// Returns how much progress is completed.
    pub fn done(&self) -> u64 {
        self.0.tick_done.load(AtomicOrdering::Acquire)
    }

    /// Returns how much progress needs to be done.
    pub fn required(&self) -> u64 {
        self.0.tick_total.load(AtomicOrdering::Acquire)
    }
}

#[derive(Default)]
struct OverallProgressInner {
    tick_done: AtomicU64,
    tick_total: AtomicU64,
}

impl OverallProgressInner {
    fn apply(&self, progress: &Progress) {
        let done = progress.done.min(progress.required);
        self.tick_done.store(done.into(), AtomicOrdering::Release);
        self.tick_total.store(progress.required.into(), AtomicOrdering::Release);
    }
}

/// Extension trait for systems that output [`Progress`] to record their progress.
pub trait ProgressTrackerSystem<Params>: IntoSystem<(), Progress, Params> {
    /// Records progress when the system finishes.
    fn track_progress<T: ProgressType>(self) -> SystemConfigs;
}

impl<S: IntoSystem<(), Progress, Params>, Params> ProgressTrackerSystem<Params> for S {
    fn track_progress<T: ProgressType>(self) -> SystemConfigs {
        self.pipe(|In(progress): In<Progress>, overall: Option<Res<OverallProgress<T>>>| {
            match overall {
                Some(overall) => {
                    progress.apply(&overall);
                },
                None => panic!("Tried to record progress, but OverallProgress<{}> wasn't in the World.",
                    std::any::type_name::<T>()),
            }
        }).in_set(ProgressTrackingSet::<T>::new())
    }
}

/// Returns a [`Condition`]-satisfying closure that will return `true` if `T` is being tracked.
pub fn currently_tracking<T: ProgressType>() -> impl Fn(Option<Res<OverallProgress<T>>>) -> bool + Clone {
    |res| { res.is_some() }
}
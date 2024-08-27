use bevy_ecs::prelude::*;
use bevy_app::{prelude::*, MainSchedulePlugin, ScheduleRunnerPlugin};
use bevy_mod_progress::*;

enum Loading {}

fn main() {
    let mut app = App::new();
    app.add_plugins(MainSchedulePlugin);
    app.add_plugins(ScheduleRunnerPlugin::default());
    app.add_plugins(ProgressTrackingPlugin::<Loading>::default());
    app.add_systems(Update, tracking_system);
    app.world_mut().spawn(Progress::<Loading>::new());
    app.observe(completion_observer);
    app.run();
}

fn tracking_system(
    mut tracked: Query<&mut Progress<Loading>>,
) {
    for mut tracker in &mut tracked {
        tracker.track(128, 128);
    }
}

fn completion_observer(
    _trigger: Trigger<Done<Loading>>,
    mut exit: EventWriter<AppExit>,
) {
    exit.send(AppExit::Success);
}
use bevy::{app::{AppExit, ScheduleRunnerPlugin}, prelude::*};
use bevy_mod_progress::*;

struct TrackerId;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ScheduleRunnerPlugin::default());
    app.add_plugins(ScheduleProgressTrackerPlugin::<TrackerId>::default());
    app.add_systems(Update, tracking_system.track_progress::<TrackerId>()
        .run_if(currently_tracking::<TrackerId>()));
    app.add_systems(Done::<TrackerId>::new(), finished_system);
    app.run();
}

fn tracking_system(
    time: Res<Time<Real>>,
) -> Progress {
    let ts = (time.elapsed_seconds() * 1000.0) as u32;
    Progress {
        done: ts.min(5000),
        required: 5000,
    }
}

fn finished_system(
    time: Res<Time<Real>>,
    mut exit: EventWriter<AppExit>,
) {
    info!("Finished at {} seconds", time.elapsed_seconds());
    exit.send(AppExit::Success);
}
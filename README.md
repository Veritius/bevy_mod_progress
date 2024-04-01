# bevy_mod_progress
Strongly typed progress tracking. Based on the (very good) [iyes_progress].

```rs
use bevy::prelude::*;
use bevy_mod_progress::*;

struct TrackerId;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ProgressTrackerPlugin::<TrackerId>::default());
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
) {
    info!("Finished at {} seconds", time.elapsed_seconds());
}
```

[iyes_progress]: https://github.com/IyesGames/iyes_progress
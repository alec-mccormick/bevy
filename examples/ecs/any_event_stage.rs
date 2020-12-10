use bevy::prelude::*;
use bevy::app::AnyEventStage;

/// This example creates a three new events & demonstrates a system that runs when any of those
/// events have fired.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_event::<EventA>()
        .add_event::<EventB>()
        .add_event::<EventC>()
        .init_resource::<EventTriggerState>()
        .add_system(event_trigger_system)
        .add_stage_after(
            stage::UPDATE,
            "event_handlers",
            AnyEventStage::<(EventA, EventB, EventC)>::default().with_system(event_listener_system)
        )
        .run();
}

#[derive(Clone, Debug)]
struct EventA;

#[derive(Clone, Debug)]
struct EventB;

#[derive(Clone, Debug)]
struct EventC;

struct EventTriggerState {
    event_timer_a: Timer,
    event_timer_b: Timer,
    event_timer_c: Timer,
}

impl Default for EventTriggerState {
    fn default() -> Self {
        EventTriggerState {
            event_timer_a: Timer::from_seconds(1.0, true),
            event_timer_b: Timer::from_seconds(0.5, true),
            event_timer_c: Timer::from_seconds(0.8, true),
        }
    }
}

// sends EventA every second, EventB every 0.5 seconds, and EventC every 0.8 seconds
fn event_trigger_system(
    time: Res<Time>,
    mut state: ResMut<EventTriggerState>,
    mut events_a: ResMut<Events<EventA>>,
    mut events_b: ResMut<Events<EventB>>,
    mut events_c: ResMut<Events<EventC>>,
) {
    if state.event_timer_a.tick(time.delta_seconds()).finished() {
        events_a.send(EventA);
    }

    if state.event_timer_b.tick(time.delta_seconds()).finished() {
        events_b.send(EventB);
    }

    if state.event_timer_c.tick(time.delta_seconds()).finished() {
        events_c.send(EventC);
    }
}

// prints events as they come in
fn event_listener_system(In((a, b, c)): In<(Option<EventA>, Option<EventB>, Option<EventC>)>) {
    println!("Received Events A: {:?} | B: {:?} | C: {:?}", a, b, c);
}
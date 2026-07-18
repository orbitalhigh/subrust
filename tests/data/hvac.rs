// hvac.rs — a house HVAC controller written as a subrust script.
//
// A user-editable control program: plain Rust that the interpreter type-checks
// and runs identically to rustc (laws L1/L2). It shows the enum + exhaustive
// `match` feature — the operating mode is a `Mode` enum, so the checker forces
// every arm to be handled and adding a mode surfaces every site that must change
// (the biggest correctness lever for logic that runs heaters and compressors
// unattended). `Option`/`Result` are on the roadmap; until then a stale or absent
// reading is signalled by `Sensor.known`, checked before any actuation.
//
// Host-injected types:
//   Env    { now_s, dow (0 = Monday), year, month, day, hour, minute, second: i64 }
//   Sensor { value: f64, age_s: i64, known: bool }   -- known=false if stale/absent
//   Relays { on: [bool; 4], age_s: i64, known: bool }
//   Health { age_s: i64, uptime_s: i64 }
// Host functions: sensor, sensor_set, relays, relays_set, relay_send, health,
//                 log, debug, debug_f64.

const TARGET_C: f64 = 21.0; // comfort setpoint
const HYSTERESIS_C: f64 = 0.5; // deadband, so the unit does not short-cycle
const NIGHT_SETBACK_C: f64 = 2.0; // lower the heat target overnight
const MAX_SENSOR_AGE_S: i64 = 120; // older readings count as unknown

// relay channels on the air handler
const CH_HEAT: i64 = 0;
const CH_COOL: i64 = 1;
const CH_FAN: i64 = 2;

#[derive(Clone, Copy)]
enum Mode {
    Off,
    Heat,
    Cool,
    Auto,
}

#[derive(Clone, Copy)]
struct State {
    mode: Mode,
    heating: bool,
    cooling: bool,
}

fn init() -> State {
    State {
        mode: Mode::Auto,
        heating: false,
        cooling: false,
    }
}

// night = 22:00..06:00 local; used to relax the heat setpoint while asleep
fn is_night(env: Env) -> bool {
    env.hour >= 22 || env.hour < 6
}

fn heat_target(env: Env) -> f64 {
    if is_night(env) {
        TARGET_C - NIGHT_SETBACK_C
    } else {
        TARGET_C
    }
}

// call for heat with hysteresis: below (target - band) when idle, below target
// once already heating
fn want_heat(temp: f64, target: f64, heating: bool) -> bool {
    if heating {
        temp < target
    } else {
        temp < target - HYSTERESIS_C
    }
}

fn want_cool(temp: f64, cooling: bool) -> bool {
    if cooling {
        temp > TARGET_C
    } else {
        temp > TARGET_C + HYSTERESIS_C
    }
}

fn all_off() {
    relay_send("hvac", CH_HEAT, false);
    relay_send("hvac", CH_COOL, false);
    relay_send("hvac", CH_FAN, false);
}

fn tick(env: Env, s: State) -> State {
    let reading = sensor("room_temp");
    // safety first: never actuate on a stale or missing reading
    if !reading.known || reading.age_s > MAX_SENSOR_AGE_S {
        all_off();
        log("room_temp unknown; holding HVAC idle");
        return State {
            mode: s.mode,
            heating: false,
            cooling: false,
        };
    }

    let temp = reading.value;
    let mut heat = false;
    let mut cool = false;

    // exhaustive over Mode — the checker rejects a forgotten arm
    match s.mode {
        Mode::Off => {}
        Mode::Heat => {
            heat = want_heat(temp, heat_target(env), s.heating);
        }
        Mode::Cool => {
            cool = want_cool(temp, s.cooling);
        }
        Mode::Auto => {
            if want_heat(temp, heat_target(env), s.heating) {
                heat = true;
            } else if want_cool(temp, s.cooling) {
                cool = true;
            }
        }
    }

    // a hard invariant: the compressor must never heat and cool at once
    assert!(!(heat && cool), "heat and cool requested together");

    relay_send("hvac", CH_HEAT, heat);
    relay_send("hvac", CH_COOL, cool);
    relay_send("hvac", CH_FAN, heat || cool);

    State {
        mode: s.mode,
        heating: heat,
        cooling: cool,
    }
}

// map a mode name to the enum (a string-literal `match` producing an enum)
fn parse_mode(name: &str) -> Mode {
    match name {
        "heat" => Mode::Heat,
        "cool" => Mode::Cool,
        "auto" => Mode::Auto,
        _ => Mode::Off,
    }
}

// `mode <off|heat|cool|auto>` — set the operating mode from a wire command
fn command(s: State, cmd: &str) -> State {
    if cmd == "mode off" {
        return State { mode: parse_mode("off"), heating: s.heating, cooling: s.cooling };
    }
    if cmd == "mode heat" {
        return State { mode: parse_mode("heat"), heating: s.heating, cooling: s.cooling };
    }
    if cmd == "mode cool" {
        return State { mode: parse_mode("cool"), heating: s.heating, cooling: s.cooling };
    }
    if cmd == "mode auto" {
        return State { mode: parse_mode("auto"), heating: s.heating, cooling: s.cooling };
    }
    log("unknown command");
    s
}


use subrust::apis::HVAC_API;
use subrust::machine::{Instance, INSTANCE_INIT};
use subrust::platform::{Platform, SrErr, SR_OK};
use subrust::{Chk, Mem, CHK_INIT, MEM_INIT};

const FUEL: u64 = 1_000_000;
const HVAC: &str = include_str!("data/hvac.rs");

struct MockHvac<'a> {
    chk: &'a Chk,
    temp: f64,
    known: bool,
    calls: Vec<String>,
}
impl<'a> MockHvac<'a> {
    fn s(&self, id: u64) -> String {
        String::from_utf8_lossy(self.chk.str_bytes(id as u32)).to_string()
    }
}
impl<'a> Platform for MockHvac<'a> {
    fn host_call(&mut self, id: u16, args: &[u64], ret: &mut [u64]) -> SrErr {
        match id {
            0 => {

                let _ = self.s(args[0]);
                ret[0] = self.temp.to_bits();
                ret[1] = 1;
                ret[2] = self.known as u64;
            }
            4 => {

                let dev = self.s(args[0]);
                self.calls
                    .push(format!("relay_send({dev}, {}, {})", args[1] as i64, args[2] != 0));
            }
            6 => self.calls.push(format!("log({})", self.s(args[0]))),
            1 | 2 | 3 | 5 | 7 | 8 => {}
            _ => return 1,
        }
        SR_OK
    }
}

fn env() -> [u64; 8] {

    [1_000_000, 1, 2026, 1, 1, 12, 0, 0]
}

fn checked() -> (Box<Mem>, Box<Chk>) {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    assert!(
        subrust::check_source(HVAC, &mut mem, &mut chk, &HVAC_API),
        "hvac.rs must type-check against HVAC_API; first diag {:#06x}",
        if mem.diag_n > 0 { mem.diags[0].code } else { 0 }
    );
    (mem, chk)
}

#[test]
fn hvac_checks_against_its_api() {
    let _ = checked();
}

#[test]
fn hvac_needs_its_api() {

    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    assert!(!subrust::check_source(HVAC, &mut mem, &mut chk, &subrust::EMPTY_API));
}

#[test]
fn hvac_tick_heats_when_cold() {
    let (mem, chk) = checked();
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = MockHvac { chk: &chk, temp: 15.0, known: true, calls: Vec::new() };

    let mut args = Vec::new();
    args.extend_from_slice(&env());
    args.extend_from_slice(&[3, 0, 0]);
    let e = subrust::call(HVAC, &mem, &chk, &mut inst, &mut host, "tick", &args, FUEL);
    assert_eq!(e, SR_OK, "trap {:#06x} at {}..{}", inst.trap_code, inst.trap_lo, inst.trap_hi);

    assert!(host.calls.contains(&"relay_send(hvac, 0, true)".to_string()), "{:?}", host.calls);
    assert!(host.calls.contains(&"relay_send(hvac, 1, false)".to_string()));
    assert!(host.calls.contains(&"relay_send(hvac, 2, true)".to_string()));
    assert_eq!(inst.result(), &[3, 1, 0]);
}

#[test]
fn hvac_idle_when_sensor_unknown() {
    let (mem, chk) = checked();
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = MockHvac { chk: &chk, temp: 0.0, known: false, calls: Vec::new() };
    let mut args = Vec::new();
    args.extend_from_slice(&env());
    args.extend_from_slice(&[3, 0, 0]);
    let e = subrust::call(HVAC, &mem, &chk, &mut inst, &mut host, "tick", &args, FUEL);
    assert_eq!(e, SR_OK);

    assert!(host.calls.contains(&"relay_send(hvac, 0, false)".to_string()), "{:?}", host.calls);
    assert!(host.calls.iter().any(|c| c.starts_with("log(")));
    assert_eq!(inst.result(), &[3, 0, 0]);
}

#[test]
fn hvac_command_sets_mode() {
    let (mem, chk) = checked();
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = MockHvac { chk: &chk, temp: 21.0, known: true, calls: Vec::new() };

    let mut cmd_id = u32::MAX;
    for k in 0..chk.str_n {
        let en = chk.strs[k];
        let lo = en.off as usize;
        if &chk.str_pool[lo..lo + en.len as usize] == b"mode heat" {
            cmd_id = k as u32;
        }
    }
    assert_ne!(cmd_id, u32::MAX);

    let mut args = Vec::new();
    args.extend_from_slice(&[3, 0, 0]);
    args.push(cmd_id as u64);
    let e = subrust::call(HVAC, &mem, &chk, &mut inst, &mut host, "command", &args, FUEL);
    assert_eq!(e, SR_OK);
    assert_eq!(inst.result(), &[1, 0, 0]);
}


use subrust::apis::TEST_API;
use subrust::machine::{Instance, INSTANCE_INIT};
use subrust::platform::{Platform, SrErr, SR_OK};
use subrust::{Chk, CHK_INIT, MEM_INIT};

const LEX_SR: &str = include_str!("../self/lex.rs");

struct PrintHost {
    out: String,
}
impl Platform for PrintHost {
    fn host_call(&mut self, id: u16, args: &[u64], _ret: &mut [u64]) -> SrErr {
        match id {
            0 => self.out.push_str(&format!("{}\n", args[0] as i64)),
            1 | 2 => self.out.push_str(&format!("{}\n", args[0])),
            4 => self.out.push_str(&format!("{}\n", args[0] != 0)),
            _ => return 1,
        }
        SR_OK
    }
}

#[test]
fn selfhost_lexer_checks_and_runs() {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(LEX_SR, &mut mem, &mut chk, &TEST_API);
    assert!(
        ok,
        "the adapted real lexer must type-check under subrust; first diag {:#06x}",
        if mem.diag_n > 0 { mem.diags[0].code } else { 0 }
    );
    let _ = &chk as &Chk;

    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost { out: String::new() };
    let e = subrust::call(LEX_SR, &mem, &chk, &mut inst, &mut host, "main", &[], 100_000_000);
    assert_eq!(e, SR_OK, "the adapted lexer must run without trapping");

    let lines: Vec<&str> = host.out.lines().collect();
    assert_eq!(lines[0], "true", "no lex errors");
    assert_eq!(lines[1], "24", "token count (23 tokens + EOF)");
    assert_eq!(lines[2], "0", "no diagnostics");

    assert_eq!(&lines[3..6], &["10", "0", "2"]);

    assert_eq!(&lines[42..45], &["2", "32", "4"]);

    assert_eq!(&lines[48..51], &["31", "38", "6"]);
}

const PARSE_SR: &str = include_str!("../self/parse.rs");

const PARSE_SAMPLE: &str = "fn f(x: i64) -> i64 { if x > 0 { x } else { 0 - x } }";

#[test]
fn selfhost_parser_checks_and_runs() {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(PARSE_SR, &mut mem, &mut chk, &TEST_API);
    assert!(
        ok,
        "the adapted real parser must type-check under subrust; first diag {:#06x} at {}",
        if mem.diag_n > 0 { mem.diags[0].code } else { 0 },
        if mem.diag_n > 0 { mem.diags[0].span.lo } else { 0 }
    );
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost { out: String::new() };
    let e = subrust::call(PARSE_SR, &mem, &chk, &mut inst, &mut host, "main", &[], 200_000_000);
    assert_eq!(e, SR_OK, "the adapted parser must run without trapping");

    let mut nmem = Box::new(MEM_INIT);
    assert!(subrust::lex::lex(PARSE_SAMPLE, &mut nmem), "native lex");
    assert!(subrust::parse::parse(PARSE_SAMPLE, &mut nmem), "native parse");

    let lines: Vec<&str> = host.out.lines().collect();
    assert_eq!(lines[0], "true", "interpreted parser lexes cleanly");
    assert_eq!(lines[1], nmem.tok_n.to_string(), "token count matches native");
    assert_eq!(lines[2], "true", "interpreted parser parses cleanly");
    assert_eq!(lines[3], "0", "no diagnostics");
    assert_eq!(lines[4], nmem.node_n.to_string(), "node count matches native parse");
    assert_eq!(lines[5], nmem.root_n.to_string(), "top-level item count matches native");

    assert_eq!(lines[6], "1", "the item is an N_FN");
    assert_eq!(nmem.node(nmem.root_first).kind, 1, "native agrees: N_FN");
}

const CHECK_SR: &str = include_str!("../self/check.rs");

#[test]
fn selfhost_checker_checks_and_runs() {
    let mut mem = Box::new(MEM_INIT);
    let mut chk = Box::new(CHK_INIT);
    let ok = subrust::check_source(CHECK_SR, &mut mem, &mut chk, &TEST_API);
    assert!(
        ok,
        "the adapted real checker must type-check under subrust; first diag {:#06x} at {}",
        if mem.diag_n > 0 { mem.diags[0].code } else { 0 },
        if mem.diag_n > 0 { mem.diags[0].span.lo } else { 0 }
    );
    let mut inst: Box<Instance> = Box::new(INSTANCE_INIT);
    let mut host = PrintHost { out: String::new() };
    let e = subrust::call(CHECK_SR, &mem, &chk, &mut inst, &mut host, "main", &[], 2_000_000_000);
    assert_eq!(e, SR_OK, "the adapted checker must run without trapping");
    let lines: Vec<&str> = host.out.lines().collect();
    assert_eq!(lines[0], "true", "interpreted checker lexes cleanly");
    assert_eq!(lines[1], "true", "interpreted checker parses cleanly");
    assert_eq!(lines[2], "true", "interpreted checker type-checks the sample");
    assert_eq!(lines[3], "0", "no diagnostics");
    assert_eq!(lines[4], "1", "one top-level item");
}

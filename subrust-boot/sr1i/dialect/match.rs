// B4 dialect corpus: match — integer/byte literal arms, multi-pattern arms
// (`1 | 2`), the wildcard `_`, match in statement position and as an expression,
// and a match returned from a function. Enums are not in the dialect (house
// style = tagged records), so there are no variant patterns. Output via putb.
fn classify(n: u64) -> u64 {
    match n {
        0 => 90,     // Z
        1 | 2 => 89, // Y — one arm, two patterns
        _ => 88,     // X
    }
}
fn main() {
    putb(classify(0)); // Z
    putb(classify(1)); // Y
    putb(classify(2)); // Y
    putb(classify(9)); // X
    // byte-value match in statement position (unit-typed arms)
    let c = b'B';
    match c {
        b'A' => putb(65),
        b'B' => putb(66), // B
        _ => putb(63),
    }
    // match as an expression, bound to a let (u64 flows into the arms)
    let r: u64 = match 3u64 {
        3 => 67, // C
        _ => 63,
    };
    putb(r);
}

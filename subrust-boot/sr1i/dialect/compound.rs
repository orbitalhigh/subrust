// B4 dialect corpus: compound assignment to a PLACE (`*p += k`, `s.field += v`,
// `arr[i] += v`). Each is a scalar read-modify-write: compute the place address,
// load the current value, apply the checked op, store back. The self-hosted
// checker uses these ~25 times (`*i += 1`, field accumulators). Output via putb.
#[derive(Clone, Copy)]
struct Acc {
    total: u64,
}
fn add_to(p: &mut u64, k: u64) {
    *p += k; // compound through a *deref place
}
fn main() {
    // *p += k through a &mut ref
    let mut x: u64 = 60;
    add_to(&mut x, 5);
    putb(x); // A (65)
    // s.field += v (field place)
    let mut a = Acc { total: 60 };
    a.total += 6;
    putb(a.total); // B (66)
    // arr[i] += v (index place, constant index)
    let mut arr = [60u64, 60, 60];
    arr[0] += 7;
    putb(arr[0]); // C (67)
    // arr[i] += v (runtime index)
    let mut i: usize = 1;
    arr[i] += 8;
    putb(arr[i]); // D (68)
    // field *= then += (checked arithmetic on a place)
    a.total *= 1;
    a.total += 3;
    putb(a.total); // E (69)
    // index += then -= (add then subtract on a place)
    arr[2] += 30;
    arr[2] -= 20;
    putb(arr[2]); // F (70)
}

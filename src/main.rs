use core::mem::MaybeUninit;
use std::ffi::CString;
use gmp_mpfr_sys::gmp;
use gmp_mpfr_sys::mpfr;
use std::ptr;

const MPREC: i64 = 128;
const ZPREC: u64 = 128;

// Naîve approach, just cast to float and do the sum that way
fn calc_n( a: u128, b: u128, amount: u128, weight1: u128, weight2: u128) -> u128 {
    return (a as f64 *
            (1_f64 - (b as f64 / (b + amount) as f64).powf(
                weight1 as f64 / weight2 as f64))) as u128
}

// Couple of utility functions for converting to and from u128 and mpfr_t.
// Do this via string representation for each; surely a less expensive way
// to do this if necessary, though gmp/mpfr does not carry with it u128 support
// so would need to do some shifting about.
fn get_mpf(inp: u128) -> mpfr::mpfr_t {
    unsafe {
        let mut m = MaybeUninit::uninit();
        mpfr::init2(m.as_mut_ptr(), MPREC);
        let mut m = m.assume_init();
        let s = CString::new(inp.to_string()).expect("char* conv failed");
        mpfr::set_str(&mut m, s.as_ptr(), 10, mpfr::rnd_t::RNDD);
        return m;
    }
}

fn get_u128(inp: mpfr::mpfr_t) -> u128 {
    unsafe {
        use std::os::raw::c_char;
        let mut z = MaybeUninit::uninit();
        gmp::mpz_init2(z.as_mut_ptr(), ZPREC);
        let mut z = z.assume_init();
        mpfr::get_z(&mut z, &inp, mpfr::rnd_t::RNDD);
        let zsc = gmp::mpz_get_str(ptr::null::<c_char>() as *mut i8, 10, &z);
        // CString should p0wn the memory IIUC
        let zs = CString::from_raw(zsc);
        let res = u128::from_str_radix(zs.to_str().expect("utf8 conv failed"), 10)
            .expect("convert final value from arbitrary precision to u128");
        gmp::mpz_clear(&mut z);
        return res;
    }
}
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// The implementation of the balancy thing using gmp/mpfr
fn calc( a: u128, b: u128, amount: u128, weight1: u128, weight2: u128) -> u128 {
    let mut am = get_mpf(a);
    let mut bm = get_mpf(b);
    let mut amountm = get_mpf(amount);
    let mut weight1m = get_mpf(weight1);
    let mut weight2m = get_mpf(weight2);
    let mut m1 = get_mpf(1);
    let res;
    unsafe {
        let mut wr = MaybeUninit::uninit();
        mpfr::init2(wr.as_mut_ptr(), MPREC);
        let mut wr = wr.assume_init();
        mpfr::div(&mut wr, &weight1m, &weight2m, mpfr::rnd_t::RNDD);

        let mut p = MaybeUninit::uninit();
        mpfr::init2(p.as_mut_ptr(), MPREC);
        let mut p = p.assume_init();
        mpfr::add(&mut p, &amountm, &bm, mpfr::rnd_t::RNDD);
        mpfr::div(&mut p, &bm, &p, mpfr::rnd_t::RNDD);
        mpfr::pow(&mut p, &p, &wr, mpfr::rnd_t::RNDD);
        mpfr::sub(&mut p, &m1, &p, mpfr::rnd_t::RNDD);
        mpfr::mul(&mut p, &am, &p, mpfr::rnd_t::RNDD);
        
        res = get_u128(p);

        mpfr::clear(&mut am);
        mpfr::clear(&mut bm);
        mpfr::clear(&mut amountm);
        mpfr::clear(&mut weight1m);
        mpfr::clear(&mut weight2m);
        mpfr::clear(&mut wr);
        mpfr::clear(&mut m1);
        mpfr::clear(&mut p);
    }

    return res;
}

fn test( a: u128, b: u128, amount: u128, weight1: u128, weight2: u128)  {
    println!("{}", calc_n(a, b, amount, weight1, weight2));
    println!("{}", calc(a, b, amount, weight1, weight2));
}

fn main() {
    test(0, 0, 0, 0, 0);
    test(1, 1, 1, 1, 1);
    test(2, 2, 2, 2, 2);
    test(1, 2, 3, 4, 5);
    test(5, 4, 3, 2, 1);
    test((1 << 100) | 1, 1000, 23, 45, 56);
}

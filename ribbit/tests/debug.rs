use core::num::NonZeroU8;

use ribbit::u2;
use ribbit::Pack as _;
use ribbit::Unpack as _;

#[derive(Copy, Clone, Debug)]
#[ribbit::pack(size = 26, debug)]
pub struct A {
    l: u16,
    m: NonZeroU8,
    c: u2,
}

#[test]
fn check() {
    let a = A {
        l: 15,
        m: NonZeroU8::new(10).unwrap(),
        c: u2::new(3),
    }
    .pack();
    assert_eq!(format!("{a:?}"), "A { l: 15, m: 10, c: 3 }");
}

#[test]
fn tuple() {
    #[derive(Copy, Clone, Debug)]
    #[ribbit::pack(size = 5, debug)]
    struct C(bool, #[ribbit(offset = 3)] u2);

    let c = C(true, u2::new(3)).pack();
    assert_eq!(format!("{c:?}"), "C(true, 3)");
}

#[test]
fn r#enum() {
    #[derive(Copy, Clone, Debug)]
    #[ribbit::pack(size = 32, debug)]
    enum Enum {
        #[ribbit(size = 0)]
        Foo,
        #[ribbit(size = 8)]
        Bar(u8),
        #[ribbit(size = 26)]
        Baz(A),
    }

    assert_eq!(format!("{:?}", Enum::Foo.pack().unpack()), "Foo");
    assert_eq!(format!("{:?}", Enum::Bar(5).pack().unpack()), "Bar(5)");
    assert_eq!(
        format!(
            "{:?}",
            <ribbit::Pack![Enum]>::new_baz(
                A {
                    l: 2,
                    m: NonZeroU8::MIN,
                    c: u2::new(3)
                }
                .pack()
            )
        ),
        "Baz(A { l: 2, m: 1, c: 3 })"
    );
}

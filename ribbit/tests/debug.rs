use core::num::NonZeroU8;

use ribbit::u2;
use ribbit::Pack as _;
use ribbit::Unpack as _;

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 26, debug)]
pub struct NamedStruct {
    l: u16,
    m: ribbit::NonZeroU8,
    c: ribbit::u2,
}

#[test]
fn named_struct() {
    let a = NamedStruct {
        l: 15,
        m: NonZeroU8::new(10).unwrap(),
        c: u2::new(3),
    }
    .pack();
    assert_eq!(format!("{a:?}"), "NamedStruct { l: 15, m: 10, c: 3 }");
}

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 5, debug)]
struct TupleStruct(bool, #[ribbit(offset = 3)] ribbit::u2);

#[test]
fn tuple_struct() {
    let c = TupleStruct(true, u2::new(3)).pack();
    assert_eq!(format!("{c:?}"), "TupleStruct(true, 3)");
}

#[derive(ribbit::Pack, Copy, Clone, Debug)]
#[ribbit(size = 32, debug)]
enum Enum {
    #[ribbit(size = 0)]
    Foo,
    #[ribbit(size = 8)]
    Bar(u8),
    #[ribbit(size = 26)]
    Baz(crate::NamedStruct),
}

#[test]
fn r#enum() {
    assert_eq!(format!("{:?}", Enum::Foo.pack().unpack()), "Foo");
    assert_eq!(format!("{:?}", Enum::Bar(5).pack().unpack()), "Bar(5)");
    assert_eq!(
        format!(
            "{:?}",
            ribbit::Packed::<Enum>::new_baz(
                NamedStruct {
                    l: 2,
                    m: NonZeroU8::MIN,
                    c: u2::new(3)
                }
                .pack()
            )
        ),
        "Baz(NamedStruct { l: 2, m: 1, c: 3 })"
    );
}

use arrayvec::ArrayString;
use arrayvec::ArrayVec;
use debugger_test::debugger_test;

#[inline(never)]
fn __break() {
    println!("Breakpoint hit");
}

#[debugger_test(
    debugger = "cdb",
    commands = r#"
.nvlist
dv

dx array
dx string

g

dx string
"#,
    expected_statements = r#"
array            : { len=0xa } [Type: arrayvec::arrayvec::ArrayVec<i32,10>]
    [<Raw View>]     [Type: arrayvec::arrayvec::ArrayVec<i32,10>]
    [len]            : 0xa [Type: unsigned int]
    [capacity]       : 10
    [0x0]            : 1 [Type: i32]
    [0x1]            : 2 [Type: i32]
    [0x2]            : 3 [Type: i32]
    [0x3]            : 4 [Type: i32]
    [0x4]            : 5 [Type: i32]
    [0x5]            : 6 [Type: i32]
    [0x6]            : 7 [Type: i32]
    [0x7]            : 8 [Type: i32]
    [0x8]            : 9 [Type: i32]
    [0x9]            : 10 [Type: i32]

string           : "foo" [Type: arrayvec::array_string::ArrayString<10>]
    [<Raw View>]     [Type: arrayvec::array_string::ArrayString<10>]
    [len]            : 0x3 [Type: unsigned int]
    [capacity]       : 10
    [0x0]            : 102 'f' [Type: char]
    [0x1]            : 111 'o' [Type: char]
    [0x2]            : 111 'o' [Type: char]

string           : "foo-bar" [Type: arrayvec::array_string::ArrayString<10>]
    [<Raw View>]     [Type: arrayvec::array_string::ArrayString<10>]
    [len]            : 0x7 [Type: unsigned int]
    [capacity]       : 10
    [0x0]            : 102 'f' [Type: char]
    [0x1]            : 111 'o' [Type: char]
    [0x2]            : 111 'o' [Type: char]
    [0x3]            : 45 '-' [Type: char]
    [0x4]            : 98 'b' [Type: char]
    [0x5]            : 97 'a' [Type: char]
    [0x6]            : 114 'r' [Type: char]
"#
)]
#[inline(never)]
fn test_debugger_visualizer() {
    let mut array = ArrayVec::<i32, 10>::new();
    for i in 0..10 {
        array.push(i + 1);
    }
    assert_eq!(&array[..], &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(array.capacity(), 10);

    let mut string = ArrayString::<10>::new();
    string.push_str("foo");
    assert_eq!(&string[..], "foo");
    assert_eq!(string.capacity(), 10);
    __break();

    string.push_str("-bar");
    assert_eq!(&string[..], "foo-bar");
    assert_eq!(string.capacity(), 10);

    let result = string.to_string();
    assert_eq!(String::from("foo-bar"), result);
    __break();
}

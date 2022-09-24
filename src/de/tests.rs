use std::collections::HashMap;
use serde::Deserialize;
use serde_bytes;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
struct MyStruct {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
enum MyEnum {
    A,
    B(bool),
    C(bool, f32),
    D { a: i32, b: i32 },
}

// #[test] 
// fn test_include() {
//     use std::path::PathBuf;
//     assert_eq!(Ok(PathBuf::from("poop")), from_str("include <>"));
// }

#[test]
fn test_empty_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct EmptyStruct1;
    assert_eq!(Ok(EmptyStruct1), from_str("EmptyStruct1"));

    #[derive(Debug, PartialEq, Deserialize)]
    struct EmptyStruct2 {}
    assert_eq!(Ok(EmptyStruct2 {}), from_str("EmptyStruct2{}"));
}

#[test]
fn test_new_type_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct NewType(i32);
    assert_eq!(Ok(NewType(42)), from_str("NewType(42)"));
    assert_eq!(Ok(NewType(33)), from_str("(33)"));
}

#[test]
fn test_unit_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct UnitStruct;
    // assert_eq!(Ok(UnitStruct), from_str("UnitStruct{}"));
    assert_eq!(Ok(UnitStruct), from_str("{}"));
}

#[test]
fn test_tuple_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct TupleStruct(f32, f32);
    assert_eq!(Ok(TupleStruct(2.0, 5.0)), from_str("TupleStruct(2,5,)"));
    assert_eq!(Ok(TupleStruct(3.0, 4.0)), from_str("(3,4)"));
}

#[test]
fn test_mapped_struct() {
    #[derive(Clone, Debug, PartialEq, Deserialize)]
    struct MappedStruct { x: HashMap<u16, u16>, y: u16 }
    let mapped_struct = MappedStruct { 
        x: HashMap::from([(4,7),(5,9)]), 
        y: 7 
    };
    
    assert_eq!(Ok(mapped_struct), 
        from_str("MappedStruct {
            x: {4:7,5:9},
            y: 7
        }"
    ));

    // Nested Cavetta Construct
    // assert_eq!(Ok(mapped_struct),
    //     from_str("MappedStruct { 
    //         x <4> 7;
    //         x <5> 9;
    //         y: 7
    //     }")
    // );
}

#[test]
fn test_vecd_struct() {
    #[derive(Clone, Debug, PartialEq, Deserialize)]
    struct VecdStruct { x: Vec<u32> }
    let vecd_struct = VecdStruct { x: vec![4, 5] };
    assert_eq!(Ok(vecd_struct),
        from_str("VecdStruct { 
            x: [4, 5],
        }")
    );

    // assert_eq!(Ok(my_struct5.clone()),
    //     from_str("MyStruct5 { 
    //         x: [4],
    //         x: [5]
    //     }")
    // );
}

#[test]
fn test_struct() {
    let my_struct = MyStruct { x: 4.0, y: 7.0 };    
    assert_eq!(Ok(my_struct), from_str("MyStruct {x:4,y:7,}"));
    assert_eq!(Ok(my_struct), from_str("{x:4,y:7}"));
    assert_eq!(Ok(my_struct),
        from_str("MyStruct { 
            <x> 4,
            <y> 7
        }")
    );
}

#[test]
fn test_map() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert((true, false), 4);
    map.insert((false, false), 123);

    assert_eq!(Ok(&map),
        from_str("{
            (true,false,):4,
            (false,false,):123,
        }").as_ref()
    );

    // Cavetta Construct
    assert_eq!(Ok(map),
        from_str("{ 
            <(true,false)> 4,
            <(false, false)> 123
        }")
    );

    // Nested map with cavetta construct (mostly maps within structs)
    // Schema: Key1 value1[<Key2> <Value2>]
    // let mut map_holder = HashMap::new();
    // let mut map2 = HashMap::new();
    // map2.insert(4, 5);
    // map_holder.insert("first", map2);
    // assert_eq!(Ok(map_holder),
    //     from_str("{
    //         first <4> 5;
    //     }")
    // );

    // let my_struct = MyStruct { x: 4.0, y: 7.0 };
    // let mut map_holder = HashMap::new();
    // let mut map2 = HashMap::new();
    // map2.insert(4, my_struct);
    // map_holder.insert("first", map2);
    // assert_eq!(Ok(map_holder),
    //     from_str("{
    //         \"first\" <4> { x:4, y:7 };
    //     }")
    // );
}

#[test]
fn test_option() {
    assert_eq!(Ok(Some(1u8)), from_str("1"));
    assert_eq!(Ok(None::<u8>), from_str("None"));
}

#[test]
fn test_enum() {
    assert_eq!(Ok(MyEnum::A), from_str("A"));
    assert_eq!(Ok(MyEnum::B(true)), from_str("B(true,)"));
    assert_eq!(Ok(MyEnum::C(true, 3.5)), from_str("C(true,3.5)"));
    assert_eq!(Ok(MyEnum::D{ a: 2, b: 3 }), from_str("D{a:2,b:3,}"));
}

#[test]
fn test_array() {
    let empty: [i32; 0] = [];
    assert_eq!(Ok(empty), from_str("()"));
    let empty_array = empty.to_vec();
    assert_eq!(Ok(empty_array), from_str("[]"));

    assert_eq!(Ok([2, 3, 4i32]), from_str("(2,3,4,)"));
    assert_eq!(Ok([2, 3, 4i32].to_vec()), from_str("[2,3,4,]"));

    assert_eq!(Ok([String::from("zme"), String::from("rald")].to_vec()), from_str("[\"zme\",rald]"));
    assert_eq!(Ok([String::from("a"), String::from("b"), String::from("he lo")].to_vec()), from_str("[a,   b,      \"he lo\"]"));
}

#[test]
fn test_string() {
    let s: String = from_str("\"わ\"").unwrap();
    assert_eq!("わ", s);
    
    let s: String = from_str("わ").unwrap();
    assert_eq!("わ", s);
}

#[test]
fn test_char() {
    assert_eq!(Ok('c'), from_str("'c'"));
    assert_eq!(Ok('c'), from_str("c"));
    assert_eq!(Ok('存'), from_str("存"));
}

#[test]
fn test_escape_char() {
    assert_eq!('\'', from_str::<char>("'\\''").unwrap());
}

#[test]
fn test_escape() {
    assert_eq!("\"Quoted\"", from_str::<String>(r#""\"Quoted\"""#).unwrap());
}

#[test]
fn test_comment() {
    assert_eq!(
        MyStruct { x: 1.0, y: 2.0 },
        from_str(
            "{
x: 1.0, # x is just 1
# There is another comment in the very next line..
# And y is indeed
y: 2.0 # 2!
}"
        )
        .unwrap()
    );
}

fn err<T>(kind: ErrorCode, line: usize, col: usize) -> Result<T> {
    Err(Error {
        code: kind,
        position: Position { line, col },
    })
}

#[test]
fn test_err_wrong_value() {
    use self::ErrorCode::*;
    use std::collections::HashMap;

    assert_eq!(from_str::<f32>("'c'"), err(ExpectedFloat, 1, 1));
    assert_eq!(from_str::<String>("'c'"), err(ExpectedString, 1, 1));
    assert_eq!(from_str::<HashMap<u32, u32>>("'c'"), err(ExpectedMap, 1, 1));
    assert_eq!(from_str::<[u8; 5]>("'c'"), err(ExpectedArray, 1, 1));
    assert_eq!(from_str::<Vec<u32>>("'c'"), err(ExpectedArray, 1, 1));
    assert_eq!(from_str::<MyEnum>("'c'"), err(ExpectedIdentifier, 1, 1));
    assert_eq!(
        from_str::<MyStruct>("'c'"),
        err(ExpectedNamedStruct("MyStruct"), 1, 1)
    );
    assert_eq!(
        from_str::<MyStruct>("NotMyStruct(x: 4, y: 2)"),
        err(
            ExpectedStructName {
                expected: "MyStruct",
                found: String::from("NotMyStruct")
            },
            1,
            1
        )
    );
    assert_eq!(from_str::<(u8, bool)>("'c'"), err(ExpectedArray, 1, 1));
    assert_eq!(from_str::<bool>("notabool"), err(ExpectedBoolean, 1, 1));

    assert_eq!(
        from_str::<MyStruct>("MyStruct{\n    x: true}"),
        err(ExpectedFloat, 2, 8)
    );
    assert_eq!(
        from_str::<MyStruct>("MyStruct{\n    x: 3.5, \n    y:}"),
        err(ExpectedFloat, 3, 7)
    );
}

#[test]
fn test_perm_ws() {
    assert_eq!(
        from_str::<MyStruct>("\nMyStruct  \t { \n x   : 3.5 , \t y\n: 4.5 \n } \t\n"),
        Ok(MyStruct { x: 3.5, y: 4.5 })
    );
}

#[test]
fn untagged() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum Untagged {
        U8(u8),
        Bool(bool),
    }

    assert_eq!(from_str::<Untagged>("true").unwrap(), Untagged::Bool(true));
    assert_eq!(from_str::<Untagged>("8").unwrap(), Untagged::U8(8));
}

#[test]
fn rename() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Foo {
        #[serde(rename = "2d")]
        D2,
        #[serde(rename = "triangle-list")]
        TriangleList,
    }
    assert_eq!(from_str::<Foo>("r#2d").unwrap(), Foo::D2);
    assert_eq!(
        from_str::<Foo>("r#triangle-list").unwrap(),
        Foo::TriangleList
    );
}

#[test]
fn forgot_apostrophes() {
    let de: Result<(i32, String)> = from_str("(4, \"Hello)");

    assert!(match de {
        Err(Error {
            code: ErrorCode::ExpectedStringEnd,
            position: _,
        }) => true,
        _ => false,
    });
}

#[test]
fn ws_tuple_newtype_variant() {
    assert_eq!(Ok(MyEnum::B(true)), from_str("B  ( \n true \n ) "));
}

#[test]
fn test_byte_stream() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct BytesStruct {
        small: Vec<u8>,
        #[serde(with = "serde_bytes")]
        large: Vec<u8>,
    }

    assert_eq!(
        Ok(BytesStruct {
            small: vec![1, 2],
            large: vec![1, 2, 3, 4]
        }),
        from_str("BytesStruct{ small:[1, 2], large:\"AQIDBA==\" }"),
    );
}

#[test]
fn test_numbers() {
    assert_eq!(
        Ok(vec![1234, 12345, 123456, 1234567, 555_555]),
        from_str("[1_234, 12_345, 1_2_3_4_5_6, 1_234_567, 5_55_55_5]"),
    );
}

#[test]
fn test_any_number_precision() {
    fn de_any_number(s: &str) -> AnyNum {
        let mut bytes = Bytes::new(s.as_bytes()).unwrap();
        bytes.any_num().unwrap()
    }

    assert_eq!(de_any_number("1"), AnyNum::U8(1));
    assert_eq!(de_any_number("+1"), AnyNum::I8(1));
    assert_eq!(de_any_number("-1"), AnyNum::I8(-1));
    assert_eq!(de_any_number("-1.0"), AnyNum::F32(-1.0));
    assert_eq!(de_any_number("1."), AnyNum::F32(1.));
    assert_eq!(de_any_number("-1."), AnyNum::F32(-1.));
    assert_eq!(de_any_number("0.3"), AnyNum::F64(0.3));
}
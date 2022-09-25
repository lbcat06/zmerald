use super::*;

use crate::error::{ Error, SpannedError, SpannedResult, Position };
use std::collections::{ HashSet, HashMap };
use serde::Deserialize;
use serde_bytes;

// Cavetta Construction
//    HashMap - <key> value,
//
// Spaga Construction
//    Vec - fieldname [value],
//          fieldname [value],
//          ...
//
// Cavetta + Spaga Construction
//    HashMap - fieldname [<key> value]
//              fieldname [<key> value]
//              ...
//
// Spagetta Construction
//    HashMap - fieldname <key> value
//              fieldname <key> value
//              ...

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
//     replace instance of `include <{path}>` with the content of the file
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
    assert_eq!(Ok(NewType(99)), from_str("NewType(99)"));
    assert_eq!(Ok(NewType(20)), from_str("(20)"));
}

#[test]
fn test_unit_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct UnitStruct;
    assert_eq!(Ok(UnitStruct), from_str("{}"));
}

#[test]
fn test_tuple_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct TupleStruct(f32, f32);
    assert_eq!(Ok(TupleStruct(1.0, 9.0)), from_str("TupleStruct(1,9,)"));
    assert_eq!(Ok(TupleStruct(6.0, 4.0)), from_str("(6,4)"));
}

#[test]
fn test_struct() {
    let my_struct = MyStruct { x: 4.0, y: 7.0 };    
    assert_eq!(Ok(my_struct), from_str("MyStruct {x:4,y:7,}"));
    assert_eq!(Ok(my_struct), from_str("{ x:4, y:7 }"));
    assert_eq!(Ok(my_struct), from_str("MyStruct { <x> 4, <y> 7 }"));
}

#[test]
fn test_vecd_struct() {
    #[derive(Clone, Debug, PartialEq, Deserialize)]
    struct VecdStruct { x: Vec<u32> }
    let vecd_struct = VecdStruct { x: vec![4, 5] };
    assert_eq!(Ok(vecd_struct.clone()),
        from_str("VecdStruct { 
            x: [4, 5],
        }")
    );

    // Spaga Construction
    assert_eq!(Ok(vecd_struct),
        from_str("VecdStruct { 
            x: [4],
            x: [5]
        }")
    );
}

#[test]
fn test_map() {
    let map = HashMap::from([
        ((true, false), 4), 
        ((false, false), 123)
    ]);

    assert_eq!(Ok(&map),
        from_str("{
            (true,false,):4,
            (false,false,):123
        }").as_ref()
    );

    // Cavetta Construction
    assert_eq!(Ok(map),
        from_str("{ 
            <(true,false)> 4,
            <(false, false)> 123
        }")
    );
}

#[test]
fn test_nested_map() {
    let map = HashMap::from([("first", HashMap::from([(4, 5), (6, 9)]))]);

    assert_eq!(Ok(map.clone()), 
        from_str("{
            first: {4:5,6:9},
        }"
    ));

    // Cavetta Construction
    assert_eq!(Ok(map.clone()),
        from_str("{
            first: { <4> 5, <6> 9 }
        }")
    );

    // Spaga Construction
    // assert_eq!(Ok(map),
    //     from_str("{
    //         first 4: 5,
    //         first 6: 9,
    //     }")
    // );

    // Cavetta + Spaga Construction
    // assert_eq!(Ok(map),
    //     from_str("{
    //         first [<4> 5],
    //         first [<6> 9],
    //     }")
    // );

    // Spagetta Construction
    // assert_eq!(Ok(map),
    //     from_str("{
    //         first <4> 5,
    //         first <6> 9,
    //     }")
    // );
}

#[test]
fn test_option() {
    assert_eq!(Ok(Some(1u8)), from_str("1"));
    assert_eq!(Ok(Some(1u8)), from_str("Some(1)"));
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
    assert_eq!(Ok(empty.to_vec()), from_str("[]"));

    assert_eq!(Ok([2, 3, 4i32]), from_str("(2,3,4,)"));
    assert_eq!(Ok([2, 3, 4i32].to_vec()), from_str("[2,3,4,]"));

    assert_eq!(Ok([String::from("zme"), String::from("rald")]), from_str("(\"zme\",rald)"));
    assert_eq!(Ok([String::from("zme"), String::from("rald")].to_vec()), from_str("[\"zme\",rald]"));
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
    assert_eq!(Ok('朦'), from_str("朦"));
    assert_eq!(Ok('\''), from_str::<char>("'\\''"));
}

#[test]
fn test_bool() {
    assert_eq!(Ok(true), from_str("true"));
    assert_eq!(Ok(false), from_str("false"));
}

#[test]
fn test_escape() {
    assert_eq!("\"Marked\"", from_str::<String>(r#""\"Marked\"""#).unwrap());
}

#[test]
fn test_comment() {
    assert_eq!(
        MyStruct { x: 1.0, y: 2.0 },
        from_str("{
            x: 1.0, # x is just 1
            # There is another comment in the very next line..
            # And y is indeed
            y: 2.0 # 2!
        } # noob")
        .unwrap()
    );
}

fn err<T>(kind: Error, line: usize, col: usize) -> SpannedResult<T> {
    Err(SpannedError {
        code: kind,
        position: Position { line, col },
    })
}

#[test]
fn test_err_wrong_value() {
    use self::Error::*;

    assert_eq!(err(ExpectedFloat, 1, 1), from_str::<f32>("'c'"));
    assert_eq!(err(ExpectedString, 1, 1), from_str::<String>("'c'"));
    // assert_eq!(err(ExpectedChar, 1, 1), from_str::<char>(r#""c""#));
    assert_eq!(err(ExpectedMap, 1, 1), from_str::<HashMap<u32, u32>>("'c'"));
    // assert_eq!(err(ExpectedMapSeparator,
    // assert_eq!(err(ExpectedMapEnd,
    assert_eq!(err(ExpectedArray, 1, 1), from_str::<[u8; 5]>("'c'"));
    assert_eq!(err(ExpectedArray, 1, 1), from_str::<Vec<u32>>("'c'"));
    // ExpectedArrayEnd,

    assert_eq!(err(ExpectedIdentifier, 1, 1), from_str::<MyEnum>("'c'"));
    assert_eq!(err(ExpectedNamedStruct("MyStruct"), 1, 1), from_str::<MyStruct>("'c'"));
    assert_eq!(err(ExpectedArray, 1, 1), from_str::<(u8, bool)>("'c'"));
    assert_eq!(err(ExpectedBoolean, 1, 1), from_str::<bool>("notabool"));
    assert_eq!(err(ExpectedFloat, 2, 8), from_str::<MyStruct>("MyStruct{\n x:    true}"));
    assert_eq!(err(ExpectedFloat, 3, 6), from_str::<MyStruct>("MyStruct{\n x: 3.5, \n   y:}"));

    // ExpectedOptionEnd,
    // ExpectedAttribute,
    // ExpectedAttributeEnd,
    // Eof,
    // assert_eq!(err(ExpectedTupleStruct,
    // assert_eq!(err(ExpectedStructEnd,
    // assert_eq!(err(ExpectedUnit,
    // assert_eq!(err(ExpectedComma,

    assert_eq!(
        from_str::<MyStruct>("NotMyStruct(x: 4, y: 2)"),
        err(
            ExpectedDifferentStructName {
                expected: "MyStruct",
                found: String::from("NotMyStruct")
            }, 1, 12
        )
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
    let de: SpannedResult<(i32, String)> = from_str("(4, \"Hello)");

    assert!(match de {
        Err(SpannedError {
            code: Error::ExpectedStringEnd,
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

#[test]
fn test_complex() {
    #[derive(Clone, Debug, PartialEq, Deserialize)]
    pub struct Layout {
        pub id: String,
        pub levels: HashMap<u16, HashSet<usize>>,    
        pub keys: HashMap<u16, Vec<Option<String>>>
    }

    let layout = Layout {
        id: String::from("bu"),
        levels: HashMap::from([(1, HashSet::from([])), (2, HashSet::from([0, 1]))]),
        keys: HashMap::from([
            (49, vec![Some(1.to_string()), Some(2.to_string()), Some(3.to_string()), Some(4.to_string())]),
            (69, vec![Some(1.to_string()), Some(2.to_string()), Some(3.to_string()), Some(4.to_string())])
        ])
    };

    assert_eq!(
        Ok(layout),
        from_str("
{
    id: bu,

    levels: {
        1: [],
        2: [ 0, 1 ],
    },

    keys: {
        49: [         1 ,         2,         3,          4 ],
        69: [         1 ,         2,         3,     4           ],
    },
}
"))
}   
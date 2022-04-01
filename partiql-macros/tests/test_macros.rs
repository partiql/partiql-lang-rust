use partiql_macros::*;

// macro converts struct S to struct H
#[my_custom_attribute]
struct S{}

#[test]
fn test_macro(){
// due to macro we have struct H in scope
    let demo=H{};
}
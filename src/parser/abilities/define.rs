use super::prelude::*;
use orion_parse::{
    define::{gal_raw_str, take_bool, take_float, take_number, take_string, take_var_ref_name},
    symbol::{symbol_assign, symbol_colon, wn_desc},
};
use orion_sec::sec::{SecFrom, SecValueObj, SecValueType, SecValueVec};
use orion_variate::vars::UpperKey;
use winnow::{combinator::separated, token::literal};

use crate::primitive::GxlObject;
pub fn gal_gxl_object(data: &mut &str) -> Result<GxlObject> {
    alt((
        take_var_ref_name.map(GxlObject::VarRef),
        gal_full_value.map(GxlObject::from),
    ))
    .parse_next(data)
}
pub fn gal_simple_value(data: &mut &str) -> Result<SecValueType> {
    alt((
        take_string.map(SecValueType::nor_from),
        take_bool.map(SecValueType::nor_from),
        take_float.map(SecValueType::nor_from),
        take_number.map(SecValueType::nor_from),
        gal_raw_str.map(SecValueType::nor_from),
    ))
    .parse_next(data)
}

pub fn gal_full_value(data: &mut &str) -> Result<SecValueType> {
    multispace0.parse_next(data)?;
    let v = alt((gal_simple_value, take_obj_value, take_vec_value_type)).parse_next(data)?;
    multispace0.parse_next(data)?;
    Ok(v)
}

pub fn gal_named_value(input: &mut &str) -> Result<(String, SecValueType)> {
    multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
        .context(wn_desc("<var-name>"))
        .parse_next(input)?;
    multispace0.parse_next(input)?;
    symbol_colon.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let val = gal_full_value
        .context(wn_desc("<var-val>"))
        .parse_next(input)?;
    multispace0(input)?;
    Ok((key.to_string(), val))
}

pub fn gal_var_assign_obj(input: &mut &str) -> Result<(String, GxlObject)> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
        .context(wn_desc("<var-name>"))
        .parse_next(input)?;
    symbol_assign.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let val = gal_gxl_object
        .context(wn_desc("<var-obj>"))
        .parse_next(input)?;
    multispace0(input)?;
    //(multispace0, alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    Ok((key.to_string(), val))
}

//task obj
// x = { a : "A", b : "B" , c : 1}
pub fn take_obj_value(data: &mut &str) -> Result<SecValueType> {
    take_value_map.parse_next(data).map(SecValueType::from)
}

pub fn take_vec_value_type(data: &mut &str) -> Result<SecValueType> {
    take_value_vec.parse_next(data).map(SecValueType::from)
}
pub fn take_value_vec(data: &mut &str) -> Result<SecValueVec> {
    let _ = multispace0.parse_next(data)?;
    literal("[").context(wn_desc("[")).parse_next(data)?;
    let _ = multispace0.parse_next(data)?;
    let items: SecValueVec = separated(0.., gal_full_value, ",").parse_next(data)?;
    literal("]").context(wn_desc("]")).parse_next(data)?;
    Ok(items)
}

pub fn take_value_map(data: &mut &str) -> Result<SecValueObj> {
    let _ = multispace0.parse_next(data)?;
    literal("{")
        .context(wn_desc("vec start"))
        .parse_next(data)?;
    let _ = multispace0.parse_next(data)?;
    let items: Vec<(String, SecValueType)> =
        separated(0.., gal_named_value, ",").parse_next(data)?;
    literal("}").parse_next(data)?;
    let mut obj = SecValueObj::new();
    items.into_iter().for_each(|(k, v)| {
        obj.insert(UpperKey::from(k), v);
    });
    Ok(obj)
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use crate::parser::inner::run_gxl;

    use super::*;

    #[test]
    fn test_assign() {
        let mut data = "data= r#\"{\"branchs\" : [{ \"name\": \"develop\" }, { \"name\" : \"release/1\"}]}\"#;";
        let (key, val) = run_gxl(gal_var_assign_obj, &mut data).assert();
        assert_eq!(key, "data".to_string());
        assert_eq!(
            val,
            GxlObject::from_val(
                r#"{"branchs" : [{ "name": "develop" }, { "name" : "release/1"}]}"#.to_string()
            )
        );
    }
    #[test]
    fn test_take_obj_value() -> Result<()> {
        // 测试空对象
        let mut input = "{}";
        assert_eq!(take_value_map(&mut input)?.len(), 0);

        // 测试单键值对对象
        let mut input = "{ key: \"value\" }";
        let obj = take_value_map(&mut input).assert();
        assert_eq!(
            obj.get(&UpperKey::from("key")).assert(),
            &SecValueType::nor_from("value".to_string())
        );

        // 测试多键值对对象
        let mut input = "{ a: 1, b: \"two\", c: true,d: 1.1 }";
        let obj = take_value_map(&mut input)?;
        assert_eq!(
            obj.get(&UpperKey::from("a")).unwrap(),
            &SecValueType::nor_from(1)
        );
        assert_eq!(
            obj.get(&UpperKey::from("b")).unwrap(),
            &SecValueType::nor_from("two".to_string())
        );
        assert_eq!(
            obj.get(&UpperKey::from("c")).unwrap(),
            &SecValueType::nor_from(true)
        );

        // 测试嵌套对象
        let mut input = "{ outer: { inner: 42 } }";
        let obj = take_value_map(&mut input).assert();
        if let SecValueType::Obj(inner) = obj.get(&UpperKey::from("outer")).assert() {
            assert_eq!(
                inner.get(&UpperKey::from("inner")).unwrap(),
                &SecValueType::nor_from(42)
            );
        } else {
            panic!("Expected nested object");
        }

        // 测试缺少闭合括号
        let mut input = "{ key: value";
        assert!(take_obj_value(&mut input).is_err());

        // 测试缺少冒号
        let mut input = "{ key value }";
        assert!(take_obj_value(&mut input).is_err());

        Ok(())
    }
    #[test]
    fn test_take_value_vec() -> Result<()> {
        use super::*;

        // 测试空列表
        let mut input = "[]";
        assert_eq!(take_value_vec(&mut input)?, Vec::<SecValueType>::new());

        // 测试单元素列表（字符串）
        let mut input = r#"["hello"]"#;
        assert_eq!(
            take_value_vec(&mut input).assert(),
            vec![SecValueType::nor_from("hello".to_string())]
        );
        let mut input = r#"["hello", "hello2"]"#;
        assert_eq!(
            take_value_vec(&mut input).assert(),
            vec![
                SecValueType::nor_from("hello".to_string()),
                SecValueType::nor_from("hello2".to_string())
            ]
        );

        // 测试多元素列表（混合类型）
        let mut input = r#"[42, "world", true]"#;
        assert_eq!(
            take_value_vec(&mut input).assert(),
            vec![
                SecValueType::nor_from(42),
                SecValueType::nor_from("world".to_string()),
                SecValueType::nor_from(true),
            ]
        );

        // 测试嵌套列表
        let mut input = r#"[[1, 2], ["a", "b"]]"#;
        assert_eq!(
            take_value_vec(&mut input)?,
            vec![
                SecValueType::List(vec![SecValueType::nor_from(1), SecValueType::nor_from(2),]),
                SecValueType::List(vec![
                    SecValueType::nor_from("a".to_string()),
                    SecValueType::nor_from("b".to_string()),
                ]),
            ]
        );

        // 测试带空格的列表
        let mut input = r#"[ 1 ,  "two" ,  false ]"#;
        assert_eq!(
            take_value_vec(&mut input)?,
            vec![
                SecValueType::nor_from(1),
                SecValueType::nor_from("two".to_string()),
                SecValueType::nor_from(false),
            ]
        );

        // 测试无效格式（缺少闭合括号）
        let mut input = r#"[1, 2"#;
        assert!(take_value_vec(&mut input).is_err());

        // 测试无效格式（缺少分隔符）
        let mut input = r#"[1 2]"#;
        assert!(take_value_vec(&mut input).is_err());

        Ok(())
    }
}

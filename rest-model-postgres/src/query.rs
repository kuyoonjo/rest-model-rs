use anyhow::{bail, Result};
use rest_model::Condition;
use serde_json::Value;
use tokio_postgres::types::ToSql;

pub fn cond_to_sql(
    cond: &Condition,
    bindings: &mut Vec<Box<dyn ToSql + Sync>>,
    seq: &mut u32,
) -> Result<String> {
    match cond {
        Condition::And(conds) => Ok(format!(
            "({})",
            conds
                .iter()
                .map(|c| cond_to_sql(c, bindings, seq))
                .collect::<Result<Vec<_>>>()?
                .join(" AND "),
        )),
        Condition::Or(conds) => Ok(format!(
            "({})",
            conds
                .iter()
                .map(|c| cond_to_sql(c, bindings, seq))
                .collect::<Result<Vec<_>>>()?
                .join(" OR "),
        )),
        Condition::Not(cond) => Ok(format!("(NOT ({}))", cond_to_sql(cond, bindings, seq)?)),
        Condition::Regex(field, value) => {
            check_invalid_chars(field)?;
            let s = *seq;
            *seq += 1;
            match value {
                Value::String(v) => {
                    bindings.push(Box::new(v.clone()));
                    Ok(format!("{} ~ ${}", field_to_key_t(&field), s))
                }
                _ => {
                    bail!("Invalid value for Regex")
                }
            }
        }
        Condition::Regexi(field, value) => {
            check_invalid_chars(field)?;
            let s = *seq;
            *seq += 1;
            match value {
                Value::String(v) => {
                    bindings.push(Box::new(v.clone()));
                    Ok(format!("{} ~* ${}", field_to_key_t(&field), s))
                }
                _ => {
                    bail!("Invalid value for Regexi")
                }
            }
        }
        Condition::Eq(field, value) => {
            check_invalid_chars(field)?;
            normal_comparison(seq, bindings, field, "=", value)
        }
        Condition::Ne(field, value) => {
            check_invalid_chars(field)?;
            normal_comparison(seq, bindings, field, "!=", value)
        }
        Condition::Gt(field, value) => {
            check_invalid_chars(field)?;
            normal_comparison(seq, bindings, field, ">", value)
        }
        Condition::Gte(field, value) => {
            check_invalid_chars(field)?;
            normal_comparison(seq, bindings, field, ">=", value)
        }
        Condition::Lt(field, value) => {
            check_invalid_chars(field)?;
            normal_comparison(seq, bindings, field, "<", value)
        }
        Condition::Lte(field, value) => {
            check_invalid_chars(field)?;
            normal_comparison(seq, bindings, field, "<=", value)
        }
        Condition::In(field, value) => {
            check_invalid_chars(field)?;
            array_comparison(seq, bindings, field, value)
        }
        Condition::Nin(field, value) => {
            check_invalid_chars(field)?;
            Ok(format!(
                "(NOT ({}))",
                array_comparison(seq, bindings, field, value)?,
            ))
        }
    }
}

pub fn sort_to_sql(sort_expr: &str) -> Result<String> {
    check_invalid_chars(sort_expr)?;
    let order_by_clauses: Vec<String> = sort_expr
        .split(|c| c == '+' || c == '-') // 按 `+` 和 `-` 拆分
        .filter(|s| !s.is_empty()) // 去除空字符串
        .map(|key| {
            let order = if sort_expr.contains(&format!("+{}", key)) {
                "ASC"
            } else {
                "DESC"
            };
            format!("{} {}", field_to_key_t(key), order) // 组合成 `key ASC/DESC`
        })
        .collect();

    if order_by_clauses.is_empty() {
        Ok("_id ASC".to_string())
    } else {
        Ok(format!("{}", order_by_clauses.join(", ")))
    }
}

fn field_to_key(field: &str) -> String {
    _field_to_key(field, false)
}

fn field_to_key_t(field: &str) -> String {
    _field_to_key(field, true)
}

fn _field_to_key(field: &str, text: bool) -> String {
    let v = field
        .split(".")
        .map(|s| format!("{}", s))
        .collect::<Vec<_>>()
        .join(",");
    let t = if text { ">" } else { "" };
    format!("data#>{}'{{{}}}'", t, v)
}

fn normal_comparison(
    seq: &mut u32,
    bindings: &mut Vec<Box<dyn ToSql + Sync>>,
    field: &str,
    op: &str,
    value: &Value,
) -> Result<String> {
    let s = *seq;
    *seq += 1;
    match value {
        Value::String(v) => match field {
            "_id" => {
                bindings.push(Box::new(v.clone()));
                Ok(format!("{} {} ${}", field, op, s))
            }
            _ => {
                bindings.push(Box::new(value.clone()));
                Ok(format!("{} {} ${}", field_to_key(field), op, s))
            }
        },
        Value::Number(v) => match field {
            "_created_at" | "_updated_at" => {
                bindings.push(Box::new(v.as_i64().unwrap()));
                Ok(format!("{} {} ${}", field, op, s))
            }
            _ => {
                bindings.push(Box::new(value.clone()));
                Ok(format!("{} {} ${}", field_to_key(field), op, s))
            }
        },
        _ => {
            bail!("Invalid value for op {}", op)
        }
    }
}

fn array_comparison(
    seq: &mut u32,
    bindings: &mut Vec<Box<dyn ToSql + Sync>>,
    field: &str,
    value: &Value,
) -> Result<String> {
    match value {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Ok("FALSE".to_string());
            } else {
                let s = *seq;
                *seq += 1;
                match field {
                    "_id" => {
                        let arr = arr.iter().map(|v| v.as_str()).collect::<Vec<_>>();
                        if arr.contains(&None) {
                            bail!("Invalid value for field {}", field)
                        } else {
                            bindings.push(Box::new(
                                arr.into_iter()
                                    .map(|v| v.unwrap().to_string())
                                    .collect::<Vec<_>>(),
                            ));
                            Ok(format!("{} = ANY(${})", field, s))
                        }
                    }
                    "_created_at" | "_updated_at" => {
                        let arr = arr.iter().map(|v| v.as_i64()).collect::<Vec<_>>();
                        if arr.contains(&None) {
                            bail!("Invalid value for field {}", field)
                        } else {
                            bindings.push(Box::new(
                                arr.into_iter().map(|v| v.unwrap()).collect::<Vec<_>>(),
                            ));
                            Ok(format!("{} = ANY(${})", field, s))
                        }
                    }
                    _ => {
                        let array_type = match arr.first() {
                            Some(Value::Number(_)) => "FLOAT8",
                            Some(Value::String(_)) => "TEXT",
                            _ => bail!("Invalid value for op IN"),
                        };

                        if array_type == "FLOAT8" {
                            let arr = arr.iter().map(|v| v.as_f64()).collect::<Vec<_>>();
                            if arr.contains(&None) {
                                bail!("Invalid value for field {}", field)
                            } else {
                                bindings.push(Box::new(
                                    arr.into_iter().map(|v| v.unwrap()).collect::<Vec<_>>(),
                                ));
                            }
                            Ok(format!(
                                "({})::{} = ANY(${})",
                                field_to_key(field),
                                array_type,
                                s
                            ))
                        } else {
                            let arr = arr.iter().map(|v| v.as_str()).collect::<Vec<_>>();
                            if arr.contains(&None) {
                                bail!("Invalid value for field {}", field)
                            } else {
                                bindings.push(Box::new(
                                    arr.into_iter()
                                        .map(|v| v.unwrap().to_string())
                                        .collect::<Vec<_>>(),
                                ));
                            }
                            Ok(format!("{} = ANY(${})", field_to_key_t(field), s))
                        }
                    }
                }
            }
        }
        _ => bail!("Invalid value for op IN"),
    }
}

fn check_invalid_chars(input: &str) -> Result<()> {
    if input.contains('\'') || input.contains('(') || input.contains(')') || input.contains(';') {
        bail!("Invalid characters in input");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rest_model::Condition;
    use serde_json::json;
    use tokio_postgres::types::ToSql;

    #[test]
    fn test_sort_to_sql() {
        assert_eq!(sort_to_sql("+name").unwrap(), "data#>>'{name}' ASC");
        assert_eq!(sort_to_sql("-age").unwrap(), "data#>>'{age}' DESC");
        assert_eq!(sort_to_sql("+name-age").unwrap(), "data#>>'{name}' ASC, data#>>'{age}' DESC");
        assert_eq!(sort_to_sql("").unwrap(), "_id ASC");
    }

    #[test]
    fn test_normal_comparison() {
        let mut seq = 1;
        let mut bindings: Vec<Box<dyn ToSql + Sync>> = Vec::new();

        let sql = normal_comparison(&mut seq, &mut bindings, "_id", "=", &json!("123")).unwrap();
        assert_eq!(sql, "_id = $1");
        assert_eq!(bindings.len(), 1);

        let sql = normal_comparison(&mut seq, &mut bindings, "_created_at", ">", &json!(123456)).unwrap();
        assert_eq!(sql, "_created_at > $2");
        assert_eq!(bindings.len(), 2);

        let sql = normal_comparison(&mut seq, &mut bindings, "age", "<", &json!(30)).unwrap();
        assert_eq!(sql, "data#>'{age}' < $3");
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_array_comparison() {
        let mut seq = 1;
        let mut bindings: Vec<Box<dyn ToSql + Sync>> = Vec::new();

        let sql = array_comparison(&mut seq, &mut bindings, "_id", &json!(["a1", "b2", "c3"])).unwrap();
        assert_eq!(sql, "_id = ANY($1)");
        assert_eq!(bindings.len(), 1);

        let sql = array_comparison(&mut seq, &mut bindings, "age", &json!([20, 25, 30])).unwrap();
        assert!(sql.contains("ANY($2)"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_cond_to_sql() {
        let mut seq = 1;
        let mut bindings: Vec<Box<dyn ToSql + Sync>> = Vec::new();

        let cond = Condition::Eq("_id".to_string(), json!("123"));
        let sql = cond_to_sql(&cond, &mut bindings, &mut seq).unwrap();
        assert_eq!(sql, "_id = $1");
        assert_eq!(bindings.len(), 1);

        let cond = Condition::In("age".to_string(), json!([25, 30, 35]));
        let sql = cond_to_sql(&cond, &mut bindings, &mut seq).unwrap();
        assert!(sql.contains("ANY($2)"));
        assert_eq!(bindings.len(), 2);

        let cond = Condition::And(vec![
            Box::new(Condition::Gt("score".to_string(), json!(80))),
            Box::new(Condition::Lt("score".to_string(), json!(100))),
        ]);
        let sql = cond_to_sql(&cond, &mut bindings, &mut seq).unwrap();
        assert_eq!(&sql, "(data#>'{score}' > $3 AND data#>'{score}' < $4)");
        assert_eq!(bindings.len(), 4);
    }

    #[test]
    fn test_regex_conditions() {
        let mut seq = 1;
        let mut bindings: Vec<Box<dyn ToSql + Sync>> = Vec::new();

        let cond = Condition::Regex("name".to_string(), json!("^J.*"));
        let sql = cond_to_sql(&cond, &mut bindings, &mut seq).unwrap();
        assert_eq!(sql, "data#>>'{name}' ~ $1");
        assert_eq!(bindings.len(), 1);

        let cond = Condition::Regexi("name".to_string(), json!("^J.*"));
        let sql = cond_to_sql(&cond, &mut bindings, &mut seq).unwrap();
        assert_eq!(sql, "data#>>'{name}' ~* $2");
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_invalid_chars() {
        assert!(check_invalid_chars("valid_field").is_ok());
        assert!(check_invalid_chars("invalid'field").is_err());
        assert!(check_invalid_chars("invalid(field)").is_err());
    }
}
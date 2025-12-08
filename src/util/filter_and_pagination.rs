use chrono::NaiveDateTime;
use user_migration::{Expr, SimpleExpr};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect, QueryTrait, Select};
use serde::{Deserialize, Serialize};
use strum::Display;
use utoipa::{IntoParams, ToSchema};

#[derive(
    Serialize, Deserialize, Debug, Display, ToSchema, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum Direction {
    DESC,
    ASC,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams, Clone, Default)]
pub struct PageQueryParam {
    pub page_num: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_direction: Option<Direction>,
    pub sort_by: Option<String>,
    pub q: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}

pub fn get_simple_expression(query_string: &str) -> Option<SimpleExpr> {
    let parts = query_string.split(':').map(|s| s.to_string()).collect::<Vec<String>>();
    if parts.len() != 3 {
        return None;
    }

    get_final_expr(&parts[0], &parts[1], &parts[2])
}

pub fn get_search_expression(modules: Vec<&str>, search_text: &str) -> Option<SimpleExpr> {
    if modules.is_empty() {
        return Some(Expr::cust("false")); // Return a default false condition if no modules are provided
    }

    let mut query_string = String::new();
    for (i, module_name) in modules.iter().enumerate() {
        if i > 0 {
            query_string.push_str(" OR "); // Use OR to check across multiple tables
        }
        query_string.push_str(&format!(
            "search_{} @@ plainto_tsquery('english', '{}')",
            module_name, search_text
        ));
    }

    if query_string.is_empty() {
        Some(Expr::cust("false")) // Return false if no valid query is constructed
    } else {
        Some(Expr::cust(query_string))
    }
}

pub fn sort_and_paginated<E>(mut select: Select<E>, param: &PageQueryParam) -> Select<E>
where
    E: EntityTrait,
{
    select = select.limit(param.page_size.unwrap_or(20));
    select = select.offset(param.page_size.unwrap_or(20) * param.page_num.unwrap_or(0));
    // if let Some(sort_by) = &param.sort_by {
    //     match param.sort_direction.unwrap_or(Direction::ASC) {
    //         Direction::DESC => return select.order_by_desc(Expr::cust(sort_by)),
    //         Direction::ASC => return select.order_by_asc(Expr::cust(sort_by)),
    //     }
    // };
    select
    // else {
    //     match param.sort_direction.unwrap_or(Direction::ASC) {
    //         Direction::DESC => select.order_by_desc(Expr::cust("created_at".to_string())),
    //         Direction::ASC => select.order_by_asc(Expr::cust("created_at".to_string())),
    //     }
    // }
}

pub fn get_final_expr(colum: &str, operation: &str, value: &str) -> Option<SimpleExpr> {
    match operation {
        "eq" => Some(Expr::cust(format!("{colum} = '{value}'")).into()),
        "lte" => Some(Expr::cust(format!("{colum} <= '{value}'")).into()),
        "lt" => Some(Expr::cust(format!("{colum} <= '{value}'")).into()),
        "ne" => Some(Expr::cust(format!("{colum} <> '{value}'")).into()),
        "gte" => Some(Expr::cust(format!("{colum} >= '{value}'")).into()),
        "gt" => Some(Expr::cust(format!("{colum} > '{value}'")).into()),
        "contains" => Some(Expr::cust(format!("{colum} >= '{value}'")).into()),
        "null" => Some(Expr::cust(format!("{colum} != '{value}'")).into()),
        "in" => Some(Expr::cust(format!("{colum} IN ({value})")).into()),
        _ => None,
    }
}

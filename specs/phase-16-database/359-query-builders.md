# Spec 359: Query Builders

## Overview
Implement type-safe, composable query builders for constructing complex SQL queries without raw string manipulation.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Core Query Builder
```rust
// src/database/query/builder.rs

use std::fmt::Write;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryBuilderError {
    #[error("Invalid query: {0}")]
    Invalid(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// SQL value for binding
#[derive(Debug, Clone)]
pub enum SqlValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<&str> for SqlValue {
    fn from(s: &str) -> Self {
        SqlValue::Text(s.to_string())
    }
}

impl From<String> for SqlValue {
    fn from(s: String) -> Self {
        SqlValue::Text(s)
    }
}

impl From<i32> for SqlValue {
    fn from(v: i32) -> Self {
        SqlValue::Int(v as i64)
    }
}

impl From<i64> for SqlValue {
    fn from(v: i64) -> Self {
        SqlValue::Int(v)
    }
}

impl From<f64> for SqlValue {
    fn from(v: f64) -> Self {
        SqlValue::Float(v)
    }
}

impl From<bool> for SqlValue {
    fn from(v: bool) -> Self {
        SqlValue::Bool(v)
    }
}

impl<T: Into<SqlValue>> From<Option<T>> for SqlValue {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => SqlValue::Null,
        }
    }
}

/// Ordering direction
#[derive(Debug, Clone, Copy, Default)]
pub enum Order {
    #[default]
    Asc,
    Desc,
}

/// Comparison operator
#[derive(Debug, Clone, Copy)]
pub enum Comparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

impl Comparison {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "<>",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
        }
    }
}

/// WHERE clause condition
#[derive(Debug, Clone)]
pub struct Condition {
    pub column: String,
    pub comparison: Comparison,
    pub value: Option<SqlValue>,
    pub values: Option<Vec<SqlValue>>,
}

impl Condition {
    pub fn eq(column: &str, value: impl Into<SqlValue>) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::Eq,
            value: Some(value.into()),
            values: None,
        }
    }

    pub fn ne(column: &str, value: impl Into<SqlValue>) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::Ne,
            value: Some(value.into()),
            values: None,
        }
    }

    pub fn lt(column: &str, value: impl Into<SqlValue>) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::Lt,
            value: Some(value.into()),
            values: None,
        }
    }

    pub fn gt(column: &str, value: impl Into<SqlValue>) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::Gt,
            value: Some(value.into()),
            values: None,
        }
    }

    pub fn like(column: &str, pattern: &str) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::Like,
            value: Some(SqlValue::Text(pattern.to_string())),
            values: None,
        }
    }

    pub fn in_list(column: &str, values: Vec<SqlValue>) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::In,
            value: None,
            values: Some(values),
        }
    }

    pub fn is_null(column: &str) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::IsNull,
            value: None,
            values: None,
        }
    }

    pub fn is_not_null(column: &str) -> Self {
        Self {
            column: column.to_string(),
            comparison: Comparison::IsNotNull,
            value: None,
            values: None,
        }
    }
}

/// Logical operator for combining conditions
#[derive(Debug, Clone, Copy)]
pub enum Logic {
    And,
    Or,
}

/// WHERE clause builder
#[derive(Debug, Clone, Default)]
pub struct WhereClause {
    conditions: Vec<(Logic, Condition)>,
    groups: Vec<(Logic, WhereClause)>,
}

impl WhereClause {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn and(mut self, condition: Condition) -> Self {
        self.conditions.push((Logic::And, condition));
        self
    }

    pub fn or(mut self, condition: Condition) -> Self {
        self.conditions.push((Logic::Or, condition));
        self
    }

    pub fn and_group(mut self, group: WhereClause) -> Self {
        self.groups.push((Logic::And, group));
        self
    }

    pub fn or_group(mut self, group: WhereClause) -> Self {
        self.groups.push((Logic::Or, group));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty() && self.groups.is_empty()
    }

    pub fn to_sql(&self, bindings: &mut Vec<SqlValue>) -> String {
        let mut parts = Vec::new();

        for (i, (logic, cond)) in self.conditions.iter().enumerate() {
            let prefix = if i == 0 { "" } else { if matches!(logic, Logic::And) { " AND " } else { " OR " } };

            let clause = match cond.comparison {
                Comparison::IsNull | Comparison::IsNotNull => {
                    format!("{}{} {}", prefix, cond.column, cond.comparison.as_sql())
                }
                Comparison::In | Comparison::NotIn => {
                    if let Some(values) = &cond.values {
                        let placeholders: Vec<_> = values.iter().map(|_| "?").collect();
                        for v in values {
                            bindings.push(v.clone());
                        }
                        format!("{}{} {} ({})", prefix, cond.column, cond.comparison.as_sql(), placeholders.join(", "))
                    } else {
                        continue;
                    }
                }
                _ => {
                    if let Some(value) = &cond.value {
                        bindings.push(value.clone());
                        format!("{}{} {} ?", prefix, cond.column, cond.comparison.as_sql())
                    } else {
                        continue;
                    }
                }
            };

            parts.push(clause);
        }

        // Handle groups
        for (logic, group) in &self.groups {
            let prefix = if parts.is_empty() { "" } else { if matches!(logic, Logic::And) { " AND " } else { " OR " } };
            let group_sql = group.to_sql(bindings);
            if !group_sql.is_empty() {
                parts.push(format!("{}({})", prefix, group_sql));
            }
        }

        parts.join("")
    }
}
```

### SELECT Query Builder
```rust
// src/database/query/select.rs

use super::builder::*;

/// SELECT query builder
#[derive(Debug, Clone)]
pub struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    distinct: bool,
    where_clause: WhereClause,
    joins: Vec<Join>,
    group_by: Vec<String>,
    having: Option<WhereClause>,
    order_by: Vec<(String, Order)>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Join {
    join_type: JoinType,
    table: String,
    on: String,
}

#[derive(Debug, Clone, Copy)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl JoinType {
    fn as_sql(&self) -> &'static str {
        match self {
            Self::Inner => "INNER JOIN",
            Self::Left => "LEFT JOIN",
            Self::Right => "RIGHT JOIN",
            Self::Full => "FULL OUTER JOIN",
        }
    }
}

impl SelectBuilder {
    pub fn from(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: vec!["*".to_string()],
            distinct: false,
            where_clause: WhereClause::new(),
            joins: Vec::new(),
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn column(mut self, col: &str) -> Self {
        if self.columns.len() == 1 && self.columns[0] == "*" {
            self.columns.clear();
        }
        self.columns.push(col.to_string());
        self
    }

    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    pub fn where_clause(mut self, clause: WhereClause) -> Self {
        self.where_clause = clause;
        self
    }

    pub fn and_where(mut self, condition: Condition) -> Self {
        self.where_clause = self.where_clause.and(condition);
        self
    }

    pub fn or_where(mut self, condition: Condition) -> Self {
        self.where_clause = self.where_clause.or(condition);
        self
    }

    pub fn inner_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Left,
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn having(mut self, clause: WhereClause) -> Self {
        self.having = Some(clause);
        self
    }

    pub fn order_by(mut self, column: &str, order: Order) -> Self {
        self.order_by.push((column.to_string(), order));
        self
    }

    pub fn order_by_asc(self, column: &str) -> Self {
        self.order_by(column, Order::Asc)
    }

    pub fn order_by_desc(self, column: &str) -> Self {
        self.order_by(column, Order::Desc)
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn paginate(self, page: i64, per_page: i64) -> Self {
        self.limit(per_page).offset((page - 1) * per_page)
    }

    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        // SELECT
        write!(sql, "SELECT ").unwrap();
        if self.distinct {
            write!(sql, "DISTINCT ").unwrap();
        }
        sql.push_str(&self.columns.join(", "));

        // FROM
        write!(sql, " FROM {}", self.table).unwrap();

        // JOINs
        for join in &self.joins {
            write!(sql, " {} {} ON {}", join.join_type.as_sql(), join.table, join.on).unwrap();
        }

        // WHERE
        if !self.where_clause.is_empty() {
            let where_sql = self.where_clause.to_sql(&mut bindings);
            write!(sql, " WHERE {}", where_sql).unwrap();
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            write!(sql, " GROUP BY {}", self.group_by.join(", ")).unwrap();
        }

        // HAVING
        if let Some(having) = &self.having {
            let having_sql = having.to_sql(&mut bindings);
            write!(sql, " HAVING {}", having_sql).unwrap();
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            let orders: Vec<_> = self.order_by
                .iter()
                .map(|(col, ord)| format!("{} {}", col, if matches!(ord, Order::Asc) { "ASC" } else { "DESC" }))
                .collect();
            write!(sql, " ORDER BY {}", orders.join(", ")).unwrap();
        }

        // LIMIT & OFFSET
        if let Some(limit) = self.limit {
            write!(sql, " LIMIT {}", limit).unwrap();
        }
        if let Some(offset) = self.offset {
            write!(sql, " OFFSET {}", offset).unwrap();
        }

        (sql, bindings)
    }
}
```

### INSERT, UPDATE, DELETE Builders
```rust
// src/database/query/mutations.rs

use super::builder::*;
use std::fmt::Write;

/// INSERT query builder
#[derive(Debug, Clone)]
pub struct InsertBuilder {
    table: String,
    columns: Vec<String>,
    values: Vec<Vec<SqlValue>>,
    on_conflict: Option<OnConflict>,
    returning: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum OnConflict {
    DoNothing,
    DoUpdate(Vec<String>),  // columns to update
    Replace,
}

impl InsertBuilder {
    pub fn into(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: Vec::new(),
            values: Vec::new(),
            on_conflict: None,
            returning: None,
        }
    }

    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn values(mut self, vals: Vec<SqlValue>) -> Self {
        self.values.push(vals);
        self
    }

    pub fn values_from_map(mut self, map: &[(&str, SqlValue)]) -> Self {
        self.columns = map.iter().map(|(k, _)| k.to_string()).collect();
        self.values.push(map.iter().map(|(_, v)| v.clone()).collect());
        self
    }

    pub fn on_conflict_do_nothing(mut self) -> Self {
        self.on_conflict = Some(OnConflict::DoNothing);
        self
    }

    pub fn on_conflict_do_update(mut self, columns: &[&str]) -> Self {
        self.on_conflict = Some(OnConflict::DoUpdate(
            columns.iter().map(|s| s.to_string()).collect()
        ));
        self
    }

    pub fn or_replace(mut self) -> Self {
        self.on_conflict = Some(OnConflict::Replace);
        self
    }

    pub fn returning(mut self, cols: &[&str]) -> Self {
        self.returning = Some(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        // INSERT
        let insert_keyword = match &self.on_conflict {
            Some(OnConflict::Replace) => "INSERT OR REPLACE",
            _ => "INSERT",
        };

        write!(sql, "{} INTO {} ({})", insert_keyword, self.table, self.columns.join(", ")).unwrap();

        // VALUES
        let mut value_groups = Vec::new();
        for row in &self.values {
            let placeholders: Vec<_> = row.iter().map(|_| "?").collect();
            value_groups.push(format!("({})", placeholders.join(", ")));
            bindings.extend(row.iter().cloned());
        }
        write!(sql, " VALUES {}", value_groups.join(", ")).unwrap();

        // ON CONFLICT
        match &self.on_conflict {
            Some(OnConflict::DoNothing) => {
                sql.push_str(" ON CONFLICT DO NOTHING");
            }
            Some(OnConflict::DoUpdate(cols)) => {
                let updates: Vec<_> = cols.iter()
                    .map(|c| format!("{} = excluded.{}", c, c))
                    .collect();
                write!(sql, " ON CONFLICT DO UPDATE SET {}", updates.join(", ")).unwrap();
            }
            _ => {}
        }

        // RETURNING
        if let Some(cols) = &self.returning {
            write!(sql, " RETURNING {}", cols.join(", ")).unwrap();
        }

        (sql, bindings)
    }
}

/// UPDATE query builder
#[derive(Debug, Clone)]
pub struct UpdateBuilder {
    table: String,
    sets: Vec<(String, SqlValue)>,
    where_clause: WhereClause,
    returning: Option<Vec<String>>,
}

impl UpdateBuilder {
    pub fn table(table: &str) -> Self {
        Self {
            table: table.to_string(),
            sets: Vec::new(),
            where_clause: WhereClause::new(),
            returning: None,
        }
    }

    pub fn set(mut self, column: &str, value: impl Into<SqlValue>) -> Self {
        self.sets.push((column.to_string(), value.into()));
        self
    }

    pub fn set_if_some<T: Into<SqlValue>>(self, column: &str, value: Option<T>) -> Self {
        match value {
            Some(v) => self.set(column, v),
            None => self,
        }
    }

    pub fn where_clause(mut self, clause: WhereClause) -> Self {
        self.where_clause = clause;
        self
    }

    pub fn and_where(mut self, condition: Condition) -> Self {
        self.where_clause = self.where_clause.and(condition);
        self
    }

    pub fn returning(mut self, cols: &[&str]) -> Self {
        self.returning = Some(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn build(self) -> Result<(String, Vec<SqlValue>), QueryBuilderError> {
        if self.sets.is_empty() {
            return Err(QueryBuilderError::MissingField("No columns to update".to_string()));
        }

        let mut sql = String::new();
        let mut bindings = Vec::new();

        // UPDATE
        write!(sql, "UPDATE {} SET ", self.table).unwrap();

        // SET
        let set_clauses: Vec<String> = self.sets
            .iter()
            .map(|(col, _)| format!("{} = ?", col))
            .collect();
        sql.push_str(&set_clauses.join(", "));
        bindings.extend(self.sets.into_iter().map(|(_, v)| v));

        // WHERE
        if !self.where_clause.is_empty() {
            let where_sql = self.where_clause.to_sql(&mut bindings);
            write!(sql, " WHERE {}", where_sql).unwrap();
        }

        // RETURNING
        if let Some(cols) = &self.returning {
            write!(sql, " RETURNING {}", cols.join(", ")).unwrap();
        }

        Ok((sql, bindings))
    }
}

/// DELETE query builder
#[derive(Debug, Clone)]
pub struct DeleteBuilder {
    table: String,
    where_clause: WhereClause,
    returning: Option<Vec<String>>,
}

impl DeleteBuilder {
    pub fn from(table: &str) -> Self {
        Self {
            table: table.to_string(),
            where_clause: WhereClause::new(),
            returning: None,
        }
    }

    pub fn where_clause(mut self, clause: WhereClause) -> Self {
        self.where_clause = clause;
        self
    }

    pub fn and_where(mut self, condition: Condition) -> Self {
        self.where_clause = self.where_clause.and(condition);
        self
    }

    pub fn returning(mut self, cols: &[&str]) -> Self {
        self.returning = Some(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        write!(sql, "DELETE FROM {}", self.table).unwrap();

        if !self.where_clause.is_empty() {
            let where_sql = self.where_clause.to_sql(&mut bindings);
            write!(sql, " WHERE {}", where_sql).unwrap();
        }

        if let Some(cols) = &self.returning {
            write!(sql, " RETURNING {}", cols.join(", ")).unwrap();
        }

        (sql, bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_builder() {
        let (sql, bindings) = SelectBuilder::from("users")
            .columns(&["id", "name", "email"])
            .and_where(Condition::eq("status", "active"))
            .and_where(Condition::gt("age", 18))
            .order_by_desc("created_at")
            .limit(10)
            .build();

        assert!(sql.contains("SELECT id, name, email FROM users"));
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("ORDER BY created_at DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_insert_builder() {
        let (sql, bindings) = InsertBuilder::into("users")
            .columns(&["name", "email"])
            .values(vec!["Alice".into(), "alice@example.com".into()])
            .build();

        assert!(sql.contains("INSERT INTO users"));
        assert!(sql.contains("VALUES"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_update_builder() {
        let (sql, bindings) = UpdateBuilder::table("users")
            .set("name", "Bob")
            .set("updated_at", "2024-01-01")
            .and_where(Condition::eq("id", 1))
            .build()
            .unwrap();

        assert!(sql.contains("UPDATE users SET"));
        assert!(sql.contains("WHERE"));
        assert_eq!(bindings.len(), 3);
    }
}
```

## Files to Create
- `src/database/query/builder.rs` - Core query builder types
- `src/database/query/select.rs` - SELECT builder
- `src/database/query/mutations.rs` - INSERT, UPDATE, DELETE builders
- `src/database/query/mod.rs` - Module exports

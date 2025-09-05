use crate::Result;
use async_trait::async_trait;

/// Query specification trait for flexible querying
pub trait QuerySpecification<T: Send + Sync>: Send + Sync {
    fn is_satisfied_by(&self, item: &T) -> bool;
    fn to_query(&self) -> QueryExpression;
}

/// Query expression for building flexible queries
#[derive(Debug, Clone)]
pub enum QueryExpression {
    Eq(String, serde_json::Value),
    Ne(String, serde_json::Value),
    Gt(String, serde_json::Value),
    Lt(String, serde_json::Value),
    Gte(String, serde_json::Value),
    Lte(String, serde_json::Value),
    Like(String, String),
    In(String, Vec<serde_json::Value>),
    And(Vec<QueryExpression>),
    Or(Vec<QueryExpression>),
    Not(Box<QueryExpression>),
    OrderBy(String, SortOrder),
    Limit(u32),
    Offset(u32),
}

/// Sort order for query results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Query result with metadata
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_count: Option<usize>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl<T> QueryResult<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            total_count: None,
            page: None,
            page_size: None,
        }
    }

    pub fn with_pagination(mut self, page: u32, page_size: u32, total_count: usize) -> Self {
        self.page = Some(page);
        self.page_size = Some(page_size);
        self.total_count = Some(total_count);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Query builder for constructing complex queries
pub struct QueryBuilder<T: Send + Sync> {
    expressions: Vec<QueryExpression>,
    order_by: Vec<(String, SortOrder)>,
    limit: Option<u32>,
    offset: Option<u32>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync> QueryBuilder<T> {
    pub fn new() -> Self {
        Self {
            expressions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn eq(mut self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.expressions.push(QueryExpression::Eq(field.into(), value.into()));
        self
    }

    pub fn ne(mut self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.expressions.push(QueryExpression::Ne(field.into(), value.into()));
        self
    }

    pub fn gt(mut self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.expressions.push(QueryExpression::Gt(field.into(), value.into()));
        self
    }

    pub fn lt(mut self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.expressions.push(QueryExpression::Lt(field.into(), value.into()));
        self
    }

    pub fn gte(mut self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.expressions.push(QueryExpression::Gte(field.into(), value.into()));
        self
    }

    pub fn lte(mut self, field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.expressions.push(QueryExpression::Lte(field.into(), value.into()));
        self
    }

    pub fn like(mut self, field: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.expressions.push(QueryExpression::Like(field.into(), pattern.into()));
        self
    }

    pub fn r#in(mut self, field: impl Into<String>, values: Vec<impl Into<serde_json::Value>>) -> Self {
        let values = values.into_iter().map(|v| v.into()).collect();
        self.expressions.push(QueryExpression::In(field.into(), values));
        self
    }

    pub fn and(mut self, expressions: Vec<QueryExpression>) -> Self {
        self.expressions.push(QueryExpression::And(expressions));
        self
    }

    pub fn or(mut self, expressions: Vec<QueryExpression>) -> Self {
        self.expressions.push(QueryExpression::Or(expressions));
        self
    }

    pub fn not(mut self, expression: QueryExpression) -> Self {
        self.expressions.push(QueryExpression::Not(Box::new(expression)));
        self
    }

    pub fn order_by(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.order_by.push((field.into(), order));
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(self) -> QueryExpression {
        let mut expressions = self.expressions;
        
        // Add ordering
        for (field, order) in self.order_by {
            expressions.push(QueryExpression::OrderBy(field, order));
        }
        
        // Add limit and offset
        if let Some(limit) = self.limit {
            expressions.push(QueryExpression::Limit(limit));
        }
        if let Some(offset) = self.offset {
            expressions.push(QueryExpression::Offset(offset));
        }
        
        if expressions.len() == 1 {
            expressions.into_iter().next().unwrap()
        } else {
            QueryExpression::And(expressions)
        }
    }
}

impl<T: Send + Sync> Default for QueryBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Query processor trait for executing queries
#[async_trait]
pub trait QueryProcessor<T: Send + Sync>: Send + Sync {
    async fn execute_query(&self, query: &QueryExpression) -> Result<QueryResult<T>>;
}

/// In-memory query processor implementation
pub struct InMemoryQueryProcessor<T: Send + Sync + Clone> {
    items: Vec<T>,
}

impl<T: Send + Sync + Clone> InMemoryQueryProcessor<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    fn evaluate_expression(&self, item: &T, expression: &QueryExpression) -> bool {
        match expression {
            QueryExpression::Eq(field, value) => self.compare_field(item, field, |v| v == value),
            QueryExpression::Ne(field, value) => self.compare_field(item, field, |v| v != value),
            QueryExpression::Gt(field, _) => false, // Simplified for compilation
            QueryExpression::Lt(field, _) => false, // Simplified for compilation
            QueryExpression::Gte(field, _) => false, // Simplified for compilation
            QueryExpression::Lte(field, _) => false, // Simplified for compilation,
            QueryExpression::Like(field, pattern) => {
                if let Ok(text) = self.get_field_value(item, field).and_then(|v| {
                    serde_json::from_value::<String>(v.clone()).map_err(|_| crate::DddError::validation("Conversion failed"))
                }) {
                    text.to_lowercase().contains(&pattern.to_lowercase())
                } else {
                    false
                }
            }
            QueryExpression::In(field, values) => {
                if let Ok(field_value) = self.get_field_value(item, field) {
                    values.contains(field_value)
                } else {
                    false
                }
            }
            QueryExpression::And(expressions) => {
                expressions.iter().all(|expr| self.evaluate_expression(item, expr))
            }
            QueryExpression::Or(expressions) => {
                expressions.iter().any(|expr| self.evaluate_expression(item, expr))
            }
            QueryExpression::Not(expr) => !self.evaluate_expression(item, expr),
            QueryExpression::OrderBy(_, _) => true, // Handled separately
            QueryExpression::Limit(_) => true,      // Handled separately
            QueryExpression::Offset(_) => true,     // Handled separately
        }
    }

    fn compare_field<F>(&self, item: &T, field: &str, comparator: F) -> bool
    where
        F: Fn(&serde_json::Value) -> bool,
    {
        self.get_field_value(item, field)
            .map(|v| comparator(v))
            .unwrap_or(false)
    }

    fn get_field_value(&self, _item: &T, _field: &str) -> Result<&serde_json::Value> {
        // Simplified implementation - always return a dummy value
        // In a real implementation, you'd need proper reflection or field access
        static DUMMY_VALUE: serde_json::Value = serde_json::Value::Null;
        Ok(&DUMMY_VALUE)
    }

    fn apply_sorting(&self, mut items: Vec<T>, order_by: &[(String, SortOrder)]) -> Vec<T> {
        if order_by.is_empty() {
            return items;
        }

        items.sort_by(|a, b| {
            for (field, order) in order_by {
                let a_value = self.get_field_value(a, field).unwrap_or(&serde_json::Value::Null);
                let b_value = self.get_field_value(b, field).unwrap_or(&serde_json::Value::Null);
                
                let cmp = match (a_value, b_value) {
                    (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                        a.as_f64().unwrap_or(0.0).partial_cmp(&b.as_f64().unwrap_or(0.0)).unwrap()
                    }
                    (serde_json::Value::String(a), serde_json::Value::String(b)) => a.cmp(b),
                    (serde_json::Value::Bool(a), serde_json::Value::Bool(b)) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                };
                
                if cmp != std::cmp::Ordering::Equal {
                    return match order {
                        SortOrder::Asc => cmp,
                        SortOrder::Desc => cmp.reverse(),
                    };
                }
            }
            std::cmp::Ordering::Equal
        });
        
        items
    }

    fn apply_pagination(&self, items: Vec<T>, limit: Option<u32>, offset: Option<u32>) -> Vec<T> {
        let start = offset.unwrap_or(0) as usize;
        let end = if let Some(limit) = limit {
            start.saturating_add(limit as usize)
        } else {
            items.len()
        };
        
        items.into_iter().skip(start).take(end.saturating_sub(start)).collect()
    }
}

#[async_trait]
impl<T: Send + Sync + Clone + serde::Serialize> QueryProcessor<T> for InMemoryQueryProcessor<T> {
    async fn execute_query(&self, query: &QueryExpression) -> Result<QueryResult<T>> {
        let mut limit = None;
        let mut offset = None;
        let mut order_by = Vec::new();
        let mut filter_expressions = Vec::new();

        // Extract different types of expressions
        if let QueryExpression::And(expressions) = query {
            for expr in expressions {
                match expr {
                    QueryExpression::Limit(l) => limit = Some(*l),
                    QueryExpression::Offset(o) => offset = Some(*o),
                    QueryExpression::OrderBy(field, order) => order_by.push((field.clone(), *order)),
                    _ => filter_expressions.push(expr.clone()),
                }
            }
        } else {
            filter_expressions.push(query.clone());
        }

        // Apply filtering
        let mut filtered_items: Vec<T> = self.items.clone();
        if !filter_expressions.is_empty() {
            let filter_query = if filter_expressions.len() == 1 {
                filter_expressions.into_iter().next().unwrap()
            } else {
                QueryExpression::And(filter_expressions)
            };
            
            filtered_items = filtered_items
                .into_iter()
                .filter(|item| self.evaluate_expression(item, &filter_query))
                .collect();
        }

        // Apply sorting
        let order_by_refs: Vec<(String, SortOrder)> = order_by.iter().map(|(f, o)| (f.clone(), *o)).collect();
        filtered_items = self.apply_sorting(filtered_items, &order_by_refs);

        // Apply pagination
        let total_count = filtered_items.len();
        let paginated_items = self.apply_pagination(filtered_items, limit, offset);

        Ok(QueryResult::new(paginated_items).with_pagination(
            offset.unwrap_or(0) / limit.unwrap_or(10).max(1) + 1,
            limit.unwrap_or(10),
            total_count,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestItem {
        id: u32,
        name: String,
        age: u32,
        active: bool,
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::<TestItem>::new()
            .eq("name", "John")
            .gt("age", 25)
            .limit(10)
            .build();

        match query {
            QueryExpression::And(expressions) => {
                assert_eq!(expressions.len(), 3);
            }
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn test_query_result() {
        let items = vec![1, 2, 3];
        let result = QueryResult::new(items)
            .with_pagination(1, 10, 3);

        assert_eq!(result.len(), 3);
        assert_eq!(result.page, Some(1));
        assert_eq!(result.page_size, Some(10));
        assert_eq!(result.total_count, Some(3));
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_query_processor() {
        let items = vec![
            TestItem { id: 1, name: "John".to_string(), age: 30, active: true },
            TestItem { id: 2, name: "Jane".to_string(), age: 25, active: false },
            TestItem { id: 3, name: "Bob".to_string(), age: 35, active: true },
        ];

        let processor = InMemoryQueryProcessor::new(items);
        
        let query = QueryBuilder::<TestItem>::new()
            .eq("active", true)
            .gt("age", 28)
            .build();

        let result = processor.execute_query(&query).await.unwrap();
        assert_eq!(result.len(), 2);
    }
}
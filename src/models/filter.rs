use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    IsNull,
    IsNotNull,
}

impl FilterOperator {
    pub fn as_str(&self) -> &str {
        match self {
            FilterOperator::Equals => "equals",
            FilterOperator::NotEquals => "not equals",
            FilterOperator::Contains => "contains",
            FilterOperator::NotContains => "not contains",
            FilterOperator::StartsWith => "starts with",
            FilterOperator::EndsWith => "ends with",
            FilterOperator::LessThan => "less than",
            FilterOperator::LessThanOrEqual => "less than or equal",
            FilterOperator::GreaterThan => "greater than",
            FilterOperator::GreaterThanOrEqual => "greater than or equal",
            FilterOperator::IsNull => "is null",
            FilterOperator::IsNotNull => "is not null",
        }
    }

    pub fn all() -> Vec<FilterOperator> {
        vec![
            FilterOperator::Equals,
            FilterOperator::NotEquals,
            FilterOperator::Contains,
            FilterOperator::NotContains,
            FilterOperator::StartsWith,
            FilterOperator::EndsWith,
            FilterOperator::LessThan,
            FilterOperator::LessThanOrEqual,
            FilterOperator::GreaterThan,
            FilterOperator::GreaterThanOrEqual,
            FilterOperator::IsNull,
            FilterOperator::IsNotNull,
        ]
    }

    pub fn needs_value(&self) -> bool {
        !matches!(self, FilterOperator::IsNull | FilterOperator::IsNotNull)
    }

    pub fn matches(&self, cell_value: &str, filter_value: &str) -> bool {
        let cell_lower = cell_value.to_lowercase();
        let filter_lower = filter_value.to_lowercase();

        match self {
            FilterOperator::Equals => cell_lower == filter_lower,
            FilterOperator::NotEquals => cell_lower != filter_lower,
            FilterOperator::Contains => cell_lower.contains(&filter_lower),
            FilterOperator::NotContains => !cell_lower.contains(&filter_lower),
            FilterOperator::StartsWith => cell_lower.starts_with(&filter_lower),
            FilterOperator::EndsWith => cell_lower.ends_with(&filter_lower),
            FilterOperator::LessThan => {
                // Try numeric comparison first
                if let (Ok(a), Ok(b)) = (cell_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    a < b
                } else {
                    cell_lower < filter_lower
                }
            }
            FilterOperator::LessThanOrEqual => {
                if let (Ok(a), Ok(b)) = (cell_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    a <= b
                } else {
                    cell_lower <= filter_lower
                }
            }
            FilterOperator::GreaterThan => {
                if let (Ok(a), Ok(b)) = (cell_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    a > b
                } else {
                    cell_lower > filter_lower
                }
            }
            FilterOperator::GreaterThanOrEqual => {
                if let (Ok(a), Ok(b)) = (cell_value.parse::<f64>(), filter_value.parse::<f64>()) {
                    a >= b
                } else {
                    cell_lower >= filter_lower
                }
            }
            FilterOperator::IsNull => cell_value.is_empty() || cell_value.eq_ignore_ascii_case("null"),
            FilterOperator::IsNotNull => !cell_value.is_empty() && !cell_value.eq_ignore_ascii_case("null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterConjunction {
    And,
    Or,
}

impl FilterConjunction {
    pub fn as_str(&self) -> &str {
        match self {
            FilterConjunction::And => "AND",
            FilterConjunction::Or => "OR",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    pub column_index: usize,
    pub operator: FilterOperator,
    pub value: String,
    pub conjunction: FilterConjunction, // Conjunction before this rule (except for first rule)
}

impl FilterRule {
    pub fn new(column_index: usize) -> Self {
        Self {
            column_index,
            operator: FilterOperator::Contains,
            value: String::new(),
            conjunction: FilterConjunction::And,
        }
    }

    pub fn matches_row(&self, row: &[String]) -> bool {
        if let Some(cell_value) = row.get(self.column_index) {
            if self.operator.needs_value() && self.value.is_empty() {
                return true; // Empty filter always matches
            }
            self.operator.matches(cell_value, &self.value)
        } else {
            false
        }
    }
}

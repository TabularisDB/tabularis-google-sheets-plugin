use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Parsed query types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum Query {
    Select(SelectQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
}

#[derive(Debug)]
pub struct SelectQuery {
    pub table: String,
    pub columns: Option<Vec<String>>, // None = SELECT *
    pub count_only: bool,
    pub where_clause: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug)]
pub struct InsertQuery {
    pub table: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct UpdateQuery {
    pub table: String,
    pub set: HashMap<String, String>,
    pub where_clause: String,
}

#[derive(Debug)]
pub struct DeleteQuery {
    pub table: String,
    pub where_clause: String,
}

// ---------------------------------------------------------------------------
// Compiled regexes
// ---------------------------------------------------------------------------

fn re_count() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r##"(?is)^SELECT\s+COUNT\s*\(\s*\*?\s*\)\s+FROM\s+(?:"([^"]+)"|`([^`]+)`|(\S+))(?:\s+WHERE\s+(.+?))?$"##
        ).unwrap()
    })
}

fn re_select() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r##"(?is)^SELECT\s+(?:DISTINCT\s+)?(.+?)\s+FROM\s+(?:"([^"]+)"|`([^`]+)`|(\S+))(?:\s+WHERE\s+(.*?))?(?:\s+ORDER\s+BY\s+.+?)?(?:\s+LIMIT\s+(\d+))?(?:\s+OFFSET\s+(\d+))?\s*$"##
        ).unwrap()
    })
}

fn re_insert() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r##"(?is)^INSERT\s+INTO\s+(?:"([^"]+)"|`([^`]+)`|(\S+))\s*\((.+?)\)\s*VALUES\s*\((.+?)\)\s*$"##
        ).unwrap()
    })
}

fn re_update() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r##"(?is)^UPDATE\s+(?:"([^"]+)"|`([^`]+)`|(\S+))\s+SET\s+(.+?)\s+WHERE\s+(.+)$"##
        ).unwrap()
    })
}

fn re_delete() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r##"(?is)^DELETE\s+FROM\s+(?:"([^"]+)"|`([^`]+)`|(\S+))\s+WHERE\s+(.+)$"##
        ).unwrap()
    })
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

pub fn parse(raw: &str) -> anyhow::Result<Query> {
    let q = raw.trim().trim_end_matches(';');

    // COUNT(*)
    if let Some(cap) = re_count().captures(q) {
        return Ok(Query::Select(SelectQuery {
            table: first_group(&cap, &[1, 2, 3]),
            columns: Some(vec!["COUNT(*)".into()]),
            count_only: true,
            where_clause: cap.get(4).map(|m| m.as_str().trim().to_string()),
            limit: None,
            offset: None,
        }));
    }

    // SELECT
    if let Some(cap) = re_select().captures(q) {
        let cols_str = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("*");
        let columns = if cols_str == "*" {
            None
        } else {
            Some(
                cols_str
                    .split(',')
                    .map(|s| unquote(s.trim()).to_string())
                    .collect(),
            )
        };
        let where_raw = cap.get(5).map(|m| m.as_str().trim());
        return Ok(Query::Select(SelectQuery {
            table: first_group(&cap, &[2, 3, 4]),
            columns,
            count_only: false,
            where_clause: where_raw.filter(|s| !s.is_empty()).map(String::from),
            limit: cap.get(6).and_then(|m| m.as_str().parse().ok()),
            offset: cap.get(7).and_then(|m| m.as_str().parse().ok()),
        }));
    }

    // INSERT
    if let Some(cap) = re_insert().captures(q) {
        let table = first_group(&cap, &[1, 2, 3]);
        let cols: Vec<&str> = cap.get(4).unwrap().as_str().split(',').collect();
        let vals_raw = cap.get(5).unwrap().as_str();
        let vals = split_values(vals_raw);
        let data = cols
            .iter()
            .zip(vals.iter())
            .map(|(c, v)| (unquote(c.trim()).to_string(), v.clone()))
            .collect();
        return Ok(Query::Insert(InsertQuery { table, data }));
    }

    // UPDATE
    if let Some(cap) = re_update().captures(q) {
        let table = first_group(&cap, &[1, 2, 3]);
        let set_clause = cap.get(4).unwrap().as_str();
        let where_clause = cap.get(5).unwrap().as_str().trim().to_string();
        let set = parse_set_clause(set_clause);
        return Ok(Query::Update(UpdateQuery {
            table,
            set,
            where_clause,
        }));
    }

    // DELETE
    if let Some(cap) = re_delete().captures(q) {
        return Ok(Query::Delete(DeleteQuery {
            table: first_group(&cap, &[1, 2, 3]),
            where_clause: cap.get(4).unwrap().as_str().trim().to_string(),
        }));
    }

    anyhow::bail!("Unsupported SQL syntax: {}", raw)
}

// ---------------------------------------------------------------------------
// WHERE evaluation
// ---------------------------------------------------------------------------

pub fn eval_where(where_clause: &str, row: &HashMap<String, String>) -> bool {
    // Split on AND (simplified: no nested parens)
    let re_and = Regex::new(r"(?i)\s+AND\s+").unwrap();
    for cond in re_and.split(where_clause) {
        let cond = cond.trim();
        if !eval_condition(cond, row) {
            return false;
        }
    }
    true
}

fn eval_condition(cond: &str, row: &HashMap<String, String>) -> bool {
    let re_cond = Regex::new(
        r##"(?i)`?"?(\w+)"?`?\s*(=|!=|<>|>=|<=|>|<|LIKE|IS\s+NULL|IS\s+NOT\s+NULL)(?:\s+(?:'([^']*)'|"([^"]*)"|(\S+)))?"##
    ).unwrap();

    let Some(cap) = re_cond.captures(cond) else {
        return true;
    };

    let col = cap.get(1).unwrap().as_str();
    let op = cap.get(2).unwrap().as_str().to_uppercase();
    let op = op.trim();
    let val = cap.get(3).or(cap.get(4)).or(cap.get(5)).map(|m| m.as_str()).unwrap_or("");
    let cell = row.get(col).map(String::as_str).unwrap_or("");

    match op {
        "IS NULL" => return cell.is_empty(),
        "IS NOT NULL" => return !cell.is_empty(),
        "LIKE" => {
            let pattern = regex::escape(val)
                .replace('%', ".*")
                .replace('_', ".");
            let re = Regex::new(&format!("(?i)^{pattern}$")).unwrap_or_else(|_| Regex::new("$^").unwrap());
            return re.is_match(cell);
        }
        _ => {}
    }

    // Try numeric comparison
    if let (Ok(cn), Ok(vn)) = (cell.parse::<f64>(), val.parse::<f64>()) {
        return match op {
            "=" => cn == vn,
            "!=" | "<>" => cn != vn,
            ">" => cn > vn,
            ">=" => cn >= vn,
            "<" => cn < vn,
            "<=" => cn <= vn,
            _ => true,
        };
    }

    // String comparison
    match op {
        "=" => cell == val,
        "!=" | "<>" => cell != val,
        ">" => cell > val,
        ">=" => cell >= val,
        "<" => cell < val,
        "<=" => cell <= val,
        _ => true,
    }
}

/// Extract `_row = N` from a WHERE clause.
pub fn extract_row_num(where_clause: &str) -> anyhow::Result<usize> {
    let re = Regex::new(r"(?i)_row\s*=\s*(\d+)").unwrap();
    re.captures(where_clause)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .ok_or_else(|| anyhow::anyhow!("WHERE clause must include '_row = <number>'"))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn first_group(cap: &regex::Captures, indices: &[usize]) -> String {
    for &i in indices {
        if let Some(m) = cap.get(i) {
            let s = m.as_str().trim();
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    String::new()
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('`') && s.ends_with('`'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Split VALUES(...) respecting single-quoted strings.
fn split_values(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in s.chars() {
        match ch {
            '\'' if !in_quotes => in_quotes = true,
            '\'' if in_quotes => in_quotes = false,
            ',' if !in_quotes => {
                result.push(current.trim().trim_matches('\'').to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        result.push(current.trim().trim_matches('\'').to_string());
    }
    result
}

fn parse_set_clause(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let re = Regex::new(r##"(?i)`?"?(\w+)"?`?\s*=\s*(?:'([^']*)'|"([^"]*)"|(\S+))"##).unwrap();
    for cap in re.captures_iter(s) {
        let key = cap.get(1).unwrap().as_str().to_string();
        let val = cap
            .get(2)
            .or(cap.get(3))
            .or(cap.get(4))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        map.insert(key, val);
    }
    map
}

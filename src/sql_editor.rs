use eframe::egui;
use std::collections::{HashSet, HashMap};

pub struct SqlEditor {
    // Autocomplete state
    show_suggestions: bool,
    suggestions: Vec<Suggestion>,
    selected_suggestion: usize,
    cursor_pos: usize,
    word_start: usize,
    // Track table aliases (alias -> table_name)
    table_aliases: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct Suggestion {
    pub text: String,
    pub kind: SuggestionKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SuggestionKind {
    Table,
    Column,
    Keyword,
}

#[derive(Debug, PartialEq, Clone)]
enum SqlToken {
    Keyword(String),
    Identifier(String),
    QuotedIdentifier(String),  // "table_name"
    StringLiteral(String),      // 'text'
    Number(String),
    Operator(String),
    Comma,
    Dot,
    Star,
    LeftParen,
    RightParen,
    Semicolon,
    Whitespace,
    Comment,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
enum ParserState {
    Start,
    InSelect,        // After SELECT, expecting columns or *
    AfterFrom,       // After FROM, expecting table name
    AfterTableName,  // After table name, expecting AS, alias, WHERE, JOIN, etc.
    AfterAs,         // After AS keyword, expecting alias identifier
    AfterJoin,       // After JOIN keyword, expecting table name
    InWhere,         // In WHERE clause, expecting columns or values
    InOrderBy,       // In ORDER BY clause, expecting columns
    InGroupBy,       // In GROUP BY clause, expecting columns
    InHaving,        // In HAVING clause, expecting aggregate conditions
}

impl SqlEditor {
    pub fn new() -> Self {
        Self {
            show_suggestions: false,
            suggestions: Vec::new(),
            selected_suggestion: 0,
            cursor_pos: 0,
            word_start: 0,
            table_aliases: HashMap::new(),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        sql: &mut String,
        tables: &[String],
        columns: &[String],
    ) -> SqlEditorResponse {
        let mut response = SqlEditorResponse {
            execute: false,
            text_changed: false,
        };

        // Handle keyboard shortcuts BEFORE creating the text edit
        // This way we can consume keys before the text edit processes them
        let mut accept_suggestion = false;
        let mut navigate_down = false;
        let mut navigate_up = false;
        let mut close_suggestions = false;
        let mut new_cursor_pos: Option<usize> = None;

        if self.show_suggestions && !self.suggestions.is_empty() {
            ui.input_mut(|i| {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                    navigate_down = true;
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                    navigate_up = true;
                }
                // Tab accepts suggestion
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Tab) {
                    accept_suggestion = true;
                }
                // Enter accepts suggestion (without Cmd/Ctrl modifier)
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                    accept_suggestion = true;
                }
                // Escape closes suggestions
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                    close_suggestions = true;
                }
            });

            // Apply navigation
            if navigate_down {
                self.selected_suggestion = (self.selected_suggestion + 1).min(self.suggestions.len() - 1);
            }
            if navigate_up {
                self.selected_suggestion = self.selected_suggestion.saturating_sub(1);
            }
            if close_suggestions {
                self.show_suggestions = false;
            }

            // Apply suggestion acceptance
            if accept_suggestion {
                if let Some(suggestion) = self.suggestions.get(self.selected_suggestion) {
                    let suggestion_text = suggestion.text.clone();
                    new_cursor_pos = Some(self.insert_suggestion(sql, &suggestion_text));
                    self.show_suggestions = false;
                    response.text_changed = true;
                }
            }
        }

        // Create the text edit with syntax highlighting
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = Self::highlight_sql(ui, string);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let text_edit = egui::TextEdit::multiline(sql)
            .desired_rows(5)
            .desired_width(f32::INFINITY)
            .code_editor()
            .layouter(&mut layouter);

        let text_response = ui.add(text_edit);

        // If we just inserted a suggestion, set the cursor position
        if let Some(pos) = new_cursor_pos {
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_response.id) {
                let ccursor = egui::text::CCursor::new(pos);
                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                state.store(ui.ctx(), text_response.id);
            }
        }

        // Track cursor position - get it from the text edit state
        let cursor_pos = if let Some(state) = egui::TextEdit::load_state(ui.ctx(), text_response.id) {
            if let Some(range) = state.cursor.char_range() {
                range.primary.index
            } else {
                sql.len()
            }
        } else {
            sql.len()
        };
        self.cursor_pos = cursor_pos;

        // Check for text changes
        if text_response.changed() {
            response.text_changed = true;
            self.update_suggestions(sql, tables, columns);
        }

        // Handle Cmd/Ctrl+Enter to execute (only when autocomplete is not showing)
        if text_response.has_focus() && !self.show_suggestions {
            ui.input(|i| {
                if i.key_pressed(egui::Key::Enter) && i.modifiers.command {
                    response.execute = true;
                }
            });
        }

        // Show autocomplete popup
        if self.show_suggestions && !self.suggestions.is_empty() {
            // Calculate popup position based on cursor location
            let popup_pos = if let Some(state) = egui::TextEdit::load_state(ui.ctx(), text_response.id) {
                if let Some(cursor_range) = state.cursor.char_range() {
                    // Get the galley (text layout) to find cursor position
                    let cursor_pos = cursor_range.primary;

                    // Get galley to calculate cursor position
                    let galley = ui.fonts(|f| {
                        let mut layout_job = Self::highlight_sql(ui, sql);
                        layout_job.wrap.max_width = text_response.rect.width();
                        f.layout_job(layout_job)
                    });

                    // Convert CCursor to Cursor using galley
                    let cursor = galley.from_ccursor(cursor_pos);

                    // Find the cursor position in the galley
                    let cursor_rect = galley.pos_from_cursor(&cursor);

                    // Position popup below the cursor
                    text_response.rect.left_top() + egui::vec2(cursor_rect.min.x, cursor_rect.max.y + 2.0)
                } else {
                    // Fallback to bottom of text editor
                    text_response.rect.left_bottom() + egui::vec2(0.0, 2.0)
                }
            } else {
                // Fallback to bottom of text editor
                text_response.rect.left_bottom() + egui::vec2(0.0, 2.0)
            };

            egui::Area::new(egui::Id::new("sql_autocomplete"))
                .fixed_pos(popup_pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width(300.0);
                        ui.set_max_height(200.0);

                        // Clone suggestions to avoid borrow checker issues
                        let suggestions_to_show: Vec<(usize, Suggestion)> = self.suggestions.iter()
                            .enumerate()
                            .take(10)
                            .map(|(i, s)| (i, s.clone()))
                            .collect();

                        let selected = self.selected_suggestion;
                        let mut clicked_suggestion: Option<String> = None;

                        let selection_color = ui.style().visuals.selection.bg_fill;

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (i, suggestion) in suggestions_to_show.iter() {
                                let is_selected = *i == selected;

                                let (icon, color) = match suggestion.kind {
                                    SuggestionKind::Table => ("📋", egui::Color32::from_rgb(100, 150, 255)),
                                    SuggestionKind::Column => ("📊", egui::Color32::from_rgb(100, 200, 100)),
                                    SuggestionKind::Keyword => ("🔑", egui::Color32::from_rgb(255, 150, 200)),
                                };

                                let button = egui::Button::new(
                                    egui::RichText::new(format!("{} {}", icon, suggestion.text))
                                        .color(color)
                                )
                                .fill(if is_selected {
                                    selection_color
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .frame(false);

                                if ui.add(button).clicked() {
                                    clicked_suggestion = Some(suggestion.text.clone());
                                }
                            }
                        });

                        if let Some(text) = clicked_suggestion {
                            let _new_pos = self.insert_suggestion(sql, &text);
                            self.show_suggestions = false;
                            // Note: cursor position won't be set immediately on click,
                            // but will be correct on next frame
                        }
                    });
                });
        }

        response
    }

    fn highlight_sql(ui: &egui::Ui, text: &str) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        let keywords = get_sql_keywords();
        let keyword_color = egui::Color32::from_rgb(255, 100, 200); // Pink/magenta
        let default_color = ui.style().visuals.text_color();

        let words: Vec<&str> = text.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_').collect();

        for word in words {
            let word_lower = word.to_lowercase();
            let trimmed = word_lower.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');

            if keywords.contains(trimmed) {
                // Keyword - highlight in pink
                job.append(
                    word,
                    0.0,
                    egui::TextFormat {
                        color: keyword_color,
                        font_id: egui::FontId::monospace(13.0),
                        ..Default::default()
                    },
                );
            } else {
                // Regular text
                job.append(
                    word,
                    0.0,
                    egui::TextFormat {
                        color: default_color,
                        font_id: egui::FontId::monospace(13.0),
                        ..Default::default()
                    },
                );
            }
        }

        job
    }

    fn extract_table_aliases(&mut self, sql: &str) {
        self.table_aliases.clear();

        let tokens = self.tokenize(sql);
        let mut i = 0;

        while i < tokens.len() {
            // Look for FROM or JOIN keywords followed by table name and optional alias
            if let SqlToken::Keyword(kw) = &tokens[i] {
                if kw == "from" || kw == "join" {
                    // Skip to next non-whitespace token (table name)
                    i += 1;
                    while i < tokens.len() && matches!(tokens[i], SqlToken::Whitespace) {
                        i += 1;
                    }

                    // Get table name
                    if i < tokens.len() {
                        let table_name = match &tokens[i] {
                            SqlToken::Identifier(name) | SqlToken::QuotedIdentifier(name) => Some(name.clone()),
                            _ => None,
                        };

                        if let Some(table) = table_name {
                            i += 1;

                            // Skip whitespace
                            while i < tokens.len() && matches!(tokens[i], SqlToken::Whitespace) {
                                i += 1;
                            }

                            // Check for AS keyword or implicit alias
                            let mut alias = None;
                            if i < tokens.len() {
                                match &tokens[i] {
                                    SqlToken::Keyword(kw) if kw == "as" => {
                                        // Skip AS and whitespace
                                        i += 1;
                                        while i < tokens.len() && matches!(tokens[i], SqlToken::Whitespace) {
                                            i += 1;
                                        }
                                        // Get alias
                                        if i < tokens.len() {
                                            if let SqlToken::Identifier(a) | SqlToken::QuotedIdentifier(a) = &tokens[i] {
                                                alias = Some(a.clone());
                                            }
                                        }
                                    }
                                    SqlToken::Identifier(a) | SqlToken::QuotedIdentifier(a) => {
                                        // Implicit alias (no AS keyword)
                                        alias = Some(a.clone());
                                    }
                                    _ => {}
                                }
                            }

                            // Store the alias mapping
                            if let Some(a) = alias {
                                self.table_aliases.insert(a.to_lowercase(), table);
                            }
                        }
                    }
                }
            }
            i += 1;
        }
    }

    fn update_suggestions(&mut self, sql: &str, tables: &[String], columns: &[String]) {
        // Extract table aliases from the SQL
        self.extract_table_aliases(sql);

        // Find the word being typed at cursor position
        let (word, word_start) = self.get_current_word(sql);
        self.word_start = word_start;

        if word.is_empty() {
            self.show_suggestions = false;
            return;
        }

        let word_lower = word.to_lowercase();

        // Check if we're typing a qualified name (e.g., "table.col" or "alias.col")
        let (qualifier, partial_name) = if let Some(dot_pos) = word.rfind('.') {
            (Some(&word[..dot_pos]), &word[dot_pos + 1..])
        } else {
            (None, word.as_str())
        };

        // Parse the SQL to understand context
        let state = self.parse_context(sql, self.cursor_pos);

        let mut suggestions = Vec::new();

        // If there's a qualifier (e.g., "table." or "alias."), suggest columns
        if let Some(qual) = qualifier {
            let partial_lower = partial_name.to_lowercase();

            // Check if the qualifier is an alias and resolve it to a table name
            let resolved_table = self.table_aliases.get(&qual.to_lowercase());

            // TODO: Ideally we would fetch columns specifically for the resolved table
            // from the database. For now, we show all available columns from the current result set.
            // This works well when the current tab has data from the same table being referenced.
            if resolved_table.is_some() || !qual.is_empty() {
                for column in columns {
                    if column.to_lowercase().starts_with(&partial_lower) {
                        suggestions.push(Suggestion {
                            text: column.clone(),
                            kind: SuggestionKind::Column,
                        });
                    }
                }
            }
        } else {
            // No qualifier, use context-based suggestions
            match state {
                ParserState::AfterFrom | ParserState::AfterJoin => {
                    // ONLY show tables after FROM or JOIN
                    for table in tables {
                        // Match against either full name (schema.table) or just table name
                        let table_lower = table.to_lowercase();
                        let matches = if table_lower.starts_with(&word_lower) {
                            true
                        } else if let Some(dot_pos) = table.rfind('.') {
                            // Also match against just the table name without schema
                            table_lower[dot_pos + 1..].starts_with(&word_lower)
                        } else {
                            false
                        };

                        if matches {
                            suggestions.push(Suggestion {
                                text: table.clone(),
                                kind: SuggestionKind::Table,
                            });
                        }
                    }
                }
            ParserState::InSelect => {
                // In SELECT clause: show columns and some keywords
                for column in columns {
                    if column.to_lowercase().starts_with(&word_lower) {
                        suggestions.push(Suggestion {
                            text: column.clone(),
                            kind: SuggestionKind::Column,
                        });
                    }
                }

                // Only show relevant keywords for SELECT (including aggregate functions)
                let select_keywords = ["distinct", "all", "as", "from", "count", "sum", "avg", "min", "max", "cast", "coalesce"];
                for keyword in select_keywords.iter() {
                    if keyword.starts_with(&word_lower) {
                        suggestions.push(Suggestion {
                            text: keyword.to_string(),
                            kind: SuggestionKind::Keyword,
                        });
                    }
                }
            }
            ParserState::InWhere | ParserState::InHaving => {
                // In WHERE/HAVING clause: show columns and comparison keywords
                for column in columns {
                    if column.to_lowercase().starts_with(&word_lower) {
                        suggestions.push(Suggestion {
                            text: column.clone(),
                            kind: SuggestionKind::Column,
                        });
                    }
                }

                // Show WHERE/HAVING-relevant keywords
                let where_keywords = ["and", "or", "not", "in", "like", "between", "is", "null"];
                for keyword in where_keywords.iter() {
                    if keyword.starts_with(&word_lower) {
                        suggestions.push(Suggestion {
                            text: keyword.to_string(),
                            kind: SuggestionKind::Keyword,
                        });
                    }
                }
            }
            ParserState::InOrderBy | ParserState::InGroupBy => {
                // In ORDER BY or GROUP BY: show columns
                for column in columns {
                    if column.to_lowercase().starts_with(&word_lower) {
                        suggestions.push(Suggestion {
                            text: column.clone(),
                            kind: SuggestionKind::Column,
                        });
                    }
                }

                // Show ordering keywords
                let order_keywords = ["asc", "desc"];
                for keyword in order_keywords.iter() {
                    if keyword.starts_with(&word_lower) {
                        suggestions.push(Suggestion {
                            text: keyword.to_string(),
                            kind: SuggestionKind::Keyword,
                        });
                    }
                }
            }
            ParserState::AfterTableName => {
                // After table name: show AS, JOIN, WHERE, ORDER BY, etc.
                let next_keywords = ["as", "where", "join", "inner", "left", "right", "on", "order", "group", "having", "limit"];
                for keyword in next_keywords.iter() {
                    if keyword.starts_with(&word_lower) {
                        suggestions.push(Suggestion {
                            text: keyword.to_string(),
                            kind: SuggestionKind::Keyword,
                        });
                    }
                }
            }
            ParserState::AfterAs => {
                // After AS keyword: don't suggest anything (alias is user-defined)
                // Could potentially suggest common alias patterns, but leave empty for now
            }
                ParserState::Start => {
                    // At start: show query keywords
                    let start_keywords = ["select", "insert", "update", "delete", "create", "alter", "drop"];
                    for keyword in start_keywords.iter() {
                        if keyword.starts_with(&word_lower) {
                            suggestions.push(Suggestion {
                                text: keyword.to_string(),
                                kind: SuggestionKind::Keyword,
                            });
                        }
                    }
                }
            }
        }

        suggestions.sort_by(|a, b| a.text.cmp(&b.text));

        self.show_suggestions = !suggestions.is_empty();
        self.suggestions = suggestions;
        self.selected_suggestion = 0;
    }

    fn tokenize(&self, sql: &str) -> Vec<SqlToken> {
        let mut tokens = Vec::new();
        let mut chars = sql.chars().peekable();
        let mut current_word = String::new();

        while let Some(c) = chars.next() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    tokens.push(SqlToken::Whitespace);
                }
                ',' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    tokens.push(SqlToken::Comma);
                }
                '.' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    tokens.push(SqlToken::Dot);
                }
                '*' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    tokens.push(SqlToken::Star);
                }
                '(' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    tokens.push(SqlToken::LeftParen);
                }
                ')' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    tokens.push(SqlToken::RightParen);
                }
                ';' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    tokens.push(SqlToken::Semicolon);
                }
                '"' => {
                    // Quoted identifier
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    let mut quoted = String::new();
                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == '"' {
                            break;
                        }
                        quoted.push(next_c);
                    }
                    tokens.push(SqlToken::QuotedIdentifier(quoted));
                }
                '\'' => {
                    // String literal
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    let mut string_lit = String::new();
                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == '\'' {
                            // Check for escaped quote ''
                            if chars.peek() == Some(&'\'') {
                                chars.next();
                                string_lit.push('\'');
                            } else {
                                break;
                            }
                        } else {
                            string_lit.push(next_c);
                        }
                    }
                    tokens.push(SqlToken::StringLiteral(string_lit));
                }
                '-' => {
                    // Check for comment --
                    if chars.peek() == Some(&'-') {
                        chars.next();
                        // Consume until end of line
                        while let Some(&next_c) = chars.peek() {
                            if next_c == '\n' {
                                break;
                            }
                            chars.next();
                        }
                        tokens.push(SqlToken::Comment);
                    } else {
                        if !current_word.is_empty() {
                            tokens.push(self.classify_token(&current_word));
                            current_word.clear();
                        }
                        let mut op = c.to_string();
                        if chars.peek() == Some(&'=') {
                            op.push(chars.next().unwrap());
                        }
                        tokens.push(SqlToken::Operator(op));
                    }
                }
                '/' => {
                    // Check for block comment /*
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        // Consume until */
                        let mut prev = ' ';
                        while let Some(next_c) = chars.next() {
                            if prev == '*' && next_c == '/' {
                                break;
                            }
                            prev = next_c;
                        }
                        tokens.push(SqlToken::Comment);
                    } else {
                        if !current_word.is_empty() {
                            tokens.push(self.classify_token(&current_word));
                            current_word.clear();
                        }
                        tokens.push(SqlToken::Operator(c.to_string()));
                    }
                }
                '=' | '<' | '>' | '!' | '+' | '%' => {
                    if !current_word.is_empty() {
                        tokens.push(self.classify_token(&current_word));
                        current_word.clear();
                    }
                    let mut op = c.to_string();
                    // Check for multi-char operators like <=, >=, !=, <>
                    if let Some(&next_c) = chars.peek() {
                        if next_c == '=' || (c == '<' && next_c == '>') {
                            op.push(chars.next().unwrap());
                        }
                    }
                    tokens.push(SqlToken::Operator(op));
                }
                _ => {
                    current_word.push(c);
                }
            }
        }

        if !current_word.is_empty() {
            tokens.push(self.classify_token(&current_word));
        }

        tokens
    }

    fn classify_token(&self, word: &str) -> SqlToken {
        let word_lower = word.to_lowercase();
        let keywords = get_sql_keywords();

        if keywords.contains(word_lower.as_str()) {
            SqlToken::Keyword(word_lower)
        } else if word.chars().all(|c| c.is_ascii_digit() || c == '.') {
            SqlToken::Number(word.to_string())
        } else {
            SqlToken::Identifier(word.to_string())
        }
    }

    fn parse_context(&self, sql: &str, cursor_pos: usize) -> ParserState {
        // Tokenize only the part before cursor
        let text_before_cursor = if cursor_pos <= sql.len() {
            &sql[..cursor_pos]
        } else {
            sql
        };

        let tokens = self.tokenize(text_before_cursor);
        let mut state = ParserState::Start;

        // Track if we should skip the last token's state transition
        // (because we might still be typing it)
        let num_tokens = tokens.len();

        for (idx, token) in tokens.iter().enumerate() {
            let is_last_token = idx == num_tokens - 1;

            match token {
                SqlToken::Whitespace | SqlToken::Comment => continue,
                SqlToken::Keyword(kw) => {
                    match kw.as_str() {
                        "select" => state = ParserState::InSelect,
                        "from" => state = ParserState::AfterFrom,
                        "where" => state = ParserState::InWhere,
                        "having" => state = ParserState::InHaving,
                        "join" => {
                            state = ParserState::AfterJoin;
                        }
                        "inner" | "left" | "right" | "outer" | "full" | "cross" => {
                            // JOIN modifier - expect JOIN keyword next
                            // Don't change state yet
                        }
                        "order" | "group" => {
                            // Look ahead for BY
                            state = match kw.as_str() {
                                "order" => ParserState::InOrderBy,
                                "group" => ParserState::InGroupBy,
                                _ => state,
                            };
                        }
                        "by" => {
                            // BY follows ORDER or GROUP
                            // State should already be set correctly
                        }
                        "as" => {
                            // After table name, AS introduces an alias
                            if state == ParserState::AfterTableName {
                                state = ParserState::AfterAs;
                            }
                            // Otherwise, AS might be in SELECT clause, don't change state
                        }
                        "on" | "and" | "or" | "in" | "like" | "between" | "is" => {
                            // These don't change the main state
                        }
                        _ => {
                            // Other keywords might indicate we're done with current clause
                        }
                    }
                }
                SqlToken::Identifier(_) | SqlToken::QuotedIdentifier(_) => {
                    // Don't transition state for the last identifier - we might still be typing it
                    if !is_last_token {
                        // After seeing an identifier in certain states, transition
                        match state {
                            ParserState::AfterFrom | ParserState::AfterJoin => {
                                state = ParserState::AfterTableName;
                            }
                            ParserState::AfterAs => {
                                // After alias, we're done with this table reference
                                state = ParserState::AfterTableName;
                            }
                            ParserState::AfterTableName => {
                                // Implicit alias (no AS keyword)
                                // Stay in AfterTableName
                            }
                            _ => {}
                        }
                    }
                }
                SqlToken::Comma => {
                    // Comma means we're continuing in the same clause
                    match state {
                        ParserState::AfterTableName => state = ParserState::AfterFrom,
                        _ => {}
                    }
                }
                SqlToken::Star | SqlToken::Dot | SqlToken::Operator(_) | SqlToken::LeftParen
                | SqlToken::RightParen | SqlToken::Semicolon | SqlToken::StringLiteral(_)
                | SqlToken::Number(_) | SqlToken::Unknown => {
                    // These don't affect state
                }
            }
        }

        state
    }

    fn get_current_word(&self, text: &str) -> (String, usize) {
        if self.cursor_pos == 0 || self.cursor_pos > text.len() {
            return (String::new(), 0);
        }

        // Ensure cursor_pos is at a valid UTF-8 boundary
        let safe_cursor_pos = if text.is_char_boundary(self.cursor_pos) {
            self.cursor_pos
        } else {
            // Find the nearest valid boundary before cursor_pos
            (0..self.cursor_pos).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0)
        };

        // Get text up to cursor
        let text_before_cursor = &text[..safe_cursor_pos];

        // Find the start of the word by going backwards through characters
        let mut start = safe_cursor_pos;
        for (i, c) in text_before_cursor.char_indices().rev() {
            if !c.is_alphanumeric() && c != '_' && c != '.' {
                // Found a non-word character, word starts after it
                start = i + c.len_utf8();
                break;
            }
            // We're at the beginning
            start = i;
        }

        let word = text[start..safe_cursor_pos].to_string();
        (word, start)
    }

    fn insert_suggestion(&mut self, text: &mut String, suggestion: &str) -> usize {
        // Ensure word_start and cursor_pos are at valid UTF-8 boundaries
        let safe_start = if text.is_char_boundary(self.word_start) {
            self.word_start
        } else {
            (0..self.word_start).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0)
        };

        let safe_end = if text.is_char_boundary(self.cursor_pos) {
            self.cursor_pos
        } else {
            (0..self.cursor_pos).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0)
        };

        // Replace the partial word with the suggestion and add a space
        let replacement = format!("{} ", suggestion);
        text.replace_range(safe_start..safe_end, &replacement);
        let new_pos = safe_start + replacement.len();
        self.cursor_pos = new_pos;
        new_pos
    }
}

pub struct SqlEditorResponse {
    pub execute: bool,
    pub text_changed: bool,
}

fn get_sql_keywords() -> HashSet<&'static str> {
    let mut keywords = HashSet::new();
    let kw = [
        "select", "from", "where", "join", "inner", "left", "right", "outer",
        "on", "and", "or", "not", "in", "like", "between", "is", "null",
        "order", "by", "group", "having", "limit", "offset", "distinct",
        "as", "case", "when", "then", "else", "end", "union", "all",
        "insert", "into", "values", "update", "set", "delete", "create",
        "table", "alter", "drop", "index", "view", "database", "schema",
        "primary", "key", "foreign", "references", "constraint", "unique",
        "default", "check", "exists", "with", "recursive", "asc", "desc",
        "count", "sum", "avg", "min", "max", "cast", "coalesce", "nullif",
        "abort", "abs", "absent", "absolute", "action", "add", "admin",
    ];

    for k in &kw {
        keywords.insert(*k);
    }

    keywords
}

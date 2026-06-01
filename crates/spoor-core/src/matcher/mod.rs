mod axios;
mod fetch;
mod graphql;
mod jquery;
mod literal;
mod location;
mod router;
mod secret;
mod source_map;
mod util;
mod window_open;
mod websocket;
mod xhr;

pub use axios::AxiosMatcher;
pub use fetch::FetchMatcher;
pub use graphql::GraphqlMatcher;
pub use jquery::JqueryMatcher;
pub use literal::LiteralCollector;
pub use location::LocationMatcher;
pub use router::RouterMatcher;
pub use secret::SecretMatcher;
pub use source_map::SourceMapMatcher;
pub use window_open::WindowOpenMatcher;
pub use websocket::WebSocketMatcher;
pub use xhr::XhrMatcher;

pub struct MatchContext<'a> {
    pub source: &'a str,
}

impl<'a> MatchContext<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let offset = offset as usize;
        let mut line = 1u32;
        let mut last_line_start = 0usize;
        for (i, b) in self.source.bytes().enumerate() {
            if i >= offset {
                break;
            }
            if b == b'\n' {
                line += 1;
                last_line_start = i + 1;
            }
        }
        let column = (offset.saturating_sub(last_line_start) + 1) as u32;
        (line, column)
    }

    pub fn snippet(&self, offset: u32, max_len: usize) -> String {
        let start = offset as usize;
        let end = (start + max_len).min(self.source.len());
        self.source.get(start..end).unwrap_or("").replace('\n', " ")
    }
}

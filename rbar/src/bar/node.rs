use crate::bar::style::Style;

pub struct Node {
    pub style: Style,
    pub content: String,
    pub children: Vec<Node>,
}

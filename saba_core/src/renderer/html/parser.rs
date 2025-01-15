use alloc::{rc::Rc, string::String, vec::Vec};
use core::{cell::RefCell, str::FromStr};

use crate::renderer::{
    dom::node::{Element, ElementKind, Node, NodeKind, Window},
    html::token::HtmlTokenizer,
};

use super::{attribute::Attribute, token::HtmlToken};

#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Rc<RefCell<Window>>,
    mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    t: HtmlTokenizer,
}

impl HtmlParser {
    pub fn new(t: HtmlTokenizer) -> Self {
        Self {
            window: Rc::new(RefCell::new(Window::new())),
            mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            t,
        }
    }

    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut cur = self.t.next();

        while let Some(ref token) = cur {
            match self.mode {
                InsertionMode::Initial => {
                    // 本書では、DOCTYPE トークンをサポートしていないため、
                    // <!doctype html> のようなトークンは文字トークンとして表される
                    // 文字トークンは無視する
                    if let HtmlToken::Char(_) = token {
                        cur = self.t.next();
                        continue;
                    }

                    self.mode = InsertionMode::BeforeHtml;
                    continue;
                }
                InsertionMode::BeforeHtml => {
                    match *token {
                        HtmlToken::Char(c) => {
                            if c == ' ' || c == '\n' {
                                cur = self.t.next();
                                continue;
                            }
                        }
                        HtmlToken::StartTag {
                            ref tag,
                            ref attributes,
                            ..
                        } => {
                            if tag == "html" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::BeforeHead;
                                cur = self.t.next();
                                continue;
                            }
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    self.insert_element("html", Vec::new());
                    self.mode = InsertionMode::BeforeHead;
                    continue;
                }
                InsertionMode::BeforeHead => {
                    match *token {
                        HtmlToken::Char(c) => {
                            if c == ' ' || c == '\n' {
                                cur = self.t.next();
                                continue;
                            }
                        }
                        HtmlToken::StartTag {
                            ref tag,
                            ref attributes,
                            ..
                        } => {
                            if tag == "head" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::InHead;
                                cur = self.t.next();
                                continue;
                            }
                        }
                        HtmlToken::Eof => return self.window.clone(),
                        _ => {}
                    }
                    self.insert_element("head", Vec::new());
                    self.mode = InsertionMode::InHead;
                    continue;
                }
                InsertionMode::InHead => {
                    match *token {
                        HtmlToken::Char(c) => {
                            if c == ' ' || c == '\n' {
                                self.insert_char(c);
                                cur = self.t.next();
                                continue;
                            }
                        }
                        HtmlToken::StartTag {
                            ref tag,
                            ref attributes,
                            ..
                        } => {
                            if tag == "style" || tag == "script" {
                                self.insert_element(tag, attributes.to_vec());
                                self.original_insertion_mode = self.mode;
                                self.mode = InsertionMode::Text;
                                cur = self.t.next();
                                continue;
                            }
                            // 仕様書には定められていないが、このブラウザは仕様をすべて実装している
                            // わけではないので、<head> が省略されている HTML 文書を扱うために
                            // 必要。これがないと <head> が省略されている HTML 文書で無限ループが
                            // 発生
                            if tag == "body" {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }
                            if let Ok(_element_kind) = ElementKind::from_str(tag) {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }
                        }
                        HtmlToken::EndTag { ref tag } => {
                            if tag == "head" {
                                self.mode = InsertionMode::AfterHead;
                                cur = self.t.next();
                                self.pop_until(ElementKind::Head);
                                continue;
                            }
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                    }
                    // <meta> や <title> などのサポートしていないタグは無視する
                    cur = self.t.next();
                    continue;
                }
                InsertionMode::AfterHead => {
                    match *token {
                        HtmlToken::Char(c) => {
                            if c == ' ' || c == '\n' {
                                self.insert_char(c);
                                cur = self.t.next();
                                continue;
                            }
                        }
                        HtmlToken::StartTag {
                            ref tag,
                            ref attributes,
                            ..
                        } => {
                            if tag == "body" {
                                self.insert_element(tag, attributes.to_vec());
                                cur = self.t.next();
                                self.mode = InsertionMode::InBody;
                                continue;
                            }
                        }
                        HtmlToken::Eof => return self.window.clone(),
                        _ => {}
                    }
                    self.insert_element("body", Vec::new());
                    self.mode = InsertionMode::InBody;
                    continue;
                }
                InsertionMode::InBody => {
                    match *token {
                        HtmlToken::StartTag {
                            ref tag,
                            ref attributes,
                            ..
                        } => match tag.as_str() {
                            "p" => {
                                self.insert_element(tag, attributes.to_vec());
                                cur = self.t.next();
                                continue;
                            }
                            "h1" | "h2" => {
                                self.insert_element(tag, attributes.to_vec());
                                cur = self.t.next();
                                continue;
                            }
                            "a" => {
                                self.insert_element(tag, attributes.to_vec());
                                cur = self.t.next();
                                continue;
                            }
                            _ => {}
                        },
                        HtmlToken::EndTag { ref tag } => {
                            match tag.as_str() {
                                "body" => {
                                    self.mode = InsertionMode::AfterBody;
                                    cur = self.t.next();
                                    if !self.contain_in_stack(ElementKind::Body) {
                                        // パースの失敗。トークンを無視する
                                        continue;
                                    }
                                    self.pop_until(ElementKind::Body);
                                    continue;
                                }
                                "html" => {
                                    if self.pop_current_node(ElementKind::Body) {
                                        self.mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html));
                                    } else {
                                        cur = self.t.next();
                                    }
                                    continue;
                                }
                                "p" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    cur = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                "h1" | "h2" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    cur = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                "a" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    cur = self.t.next();
                                    self.pop_until(element_kind);
                                    continue;
                                }
                                _ => cur = self.t.next(),
                            }
                        }
                        HtmlToken::Char(c) => {
                            self.insert_char(c);
                            cur = self.t.next();
                            continue;
                        }
                        HtmlToken::Eof => return self.window.clone(),
                    }
                }
                InsertionMode::AfterBody => {
                    match *token {
                        HtmlToken::Char(_) => {
                            cur = self.t.next();
                            continue;
                        }
                        HtmlToken::EndTag { ref tag } => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                cur = self.t.next();
                                continue;
                            }
                        }
                        HtmlToken::Eof => return self.window.clone(),
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                }
                InsertionMode::AfterAfterBody => {
                    match *token {
                        HtmlToken::Char(_) => {
                            cur = self.t.next();
                            continue;
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    // パースの失敗
                    self.mode = InsertionMode::InBody;
                }
                _ => {}
            }
        }

        self.window.clone()
    }

    fn create_element(&self, tag: &str, attributes: Vec<Attribute>) -> Node {
        Node::new(NodeKind::Element(Element::new(tag, attributes)))
    }

    fn insert_element(&mut self, tag: &str, attributes: Vec<Attribute>) {
        let window = self.window.borrow();
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => window.document(),
        };

        let node = Rc::new(RefCell::new(self.create_element(tag, attributes)));

        if current.borrow().first_child().is_some() {
            let mut last_sibling = current.borrow().first_child().unwrap();
            loop {
                let Some(next) = last_sibling.borrow().next_sibling() else {
                    break;
                };
                last_sibling = next;
            }

            last_sibling
                .borrow_mut()
                .set_next_sibling(Some(node.clone()));
            node.borrow_mut()
                .set_previous_sibling(Rc::downgrade(&last_sibling));
        } else {
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        node.borrow_mut().set_parent(Rc::downgrade(&current));

        self.stack_of_open_elements.push(node);
    }

    fn pop_current_node(&mut self, element_kind: ElementKind) -> bool {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => return false,
        };

        if current.borrow().element_kind() == Some(element_kind) {
            self.stack_of_open_elements.pop();
            true
        } else {
            false
        }
    }

    fn contain_in_stack(&mut self, element_kind: ElementKind) -> bool {
        self.stack_of_open_elements
            .iter()
            .any(|node| node.borrow().element_kind() == Some(element_kind))
    }

    fn pop_until(&mut self, element_kind: ElementKind) {
        assert!(
            self.contain_in_stack(element_kind),
            "stack doesn't have an element {:?}",
            element_kind
        );

        loop {
            let current = match self.stack_of_open_elements.pop() {
                Some(n) => n,
                None => return,
            };

            if current.borrow().element_kind() == Some(element_kind) {
                return;
            }
        }
    }

    fn create_char(&self, c: char) -> Node {
        let mut s = String::new();
        s.push(c);
        Node::new(NodeKind::Text(s))
    }

    fn insert_char(&mut self, c: char) {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => return,
        };

        // 現在参照しているノードがテキストノードの場合、そのノードに文字を追加する
        if let NodeKind::Text(ref mut s) = current.borrow_mut().kind {
            s.push(c);
            return;
        }

        // 改行文字や空白文字の時はテキストノードを追加しない
        if c == '\n' || c == ' ' {
            return;
        }

        let node = Rc::new(RefCell::new(self.create_char(c)));

        let first_child = current.borrow().first_child();
        if first_child.is_some() {
            let first_child = first_child.unwrap();
            first_child
                .borrow_mut()
                .set_next_sibling(Some(node.clone()));
            node.borrow_mut()
                .set_previous_sibling(Rc::downgrade(&first_child))
        } else {
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        node.borrow_mut().set_parent(Rc::downgrade(current));

        self.stack_of_open_elements.push(node);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody,
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use super::*;

    #[test]
    fn test_empty() {
        let html = "".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let expected = Rc::new(RefCell::new(Node::new(NodeKind::Document)));

        assert_eq!(expected, window.borrow().document());
    }

    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document,
        );

        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new(),
            ))))),
            html,
        );

        let head = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of html");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "head",
                Vec::new(),
            ))))),
            head,
        );

        let body = head
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new(),
            ))))),
            body,
        );
    }

    #[test]
    fn test_text() {
        let html = "<html><head></head><body>text</body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document,
        );

        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new(),
            ))))),
            html,
        );

        let body = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new(),
            ))))),
            body,
        );

        let text = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text,
        );
    }

    #[test]
    fn test_multiple_nodes() {
        let html = "<html><head></head><body><p><a foo=bar>text</a></p></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();

        let body = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .first_child()
            .expect("failed to get a frist child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body,
        );

        let p = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of body");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "p",
                Vec::new()
            ))))),
            p,
        );

        let mut attr = Attribute::new();
        attr.add_char('f', true);
        attr.add_char('o', true);
        attr.add_char('o', true);
        attr.add_char('b', false);
        attr.add_char('a', false);
        attr.add_char('r', false);
        let a = p
            .borrow()
            .first_child()
            .expect("failed to get a first child of p");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "a",
                vec![attr]
            ))))),
            a,
        );

        let text = a
            .borrow()
            .first_child()
            .expect("failed to get a first child of a");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text,
        );
    }
}

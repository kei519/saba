use core::cell::RefCell;

use alloc::{format, rc::Rc, string::String};

use crate::renderer::dom::node::Node;

pub fn convert_dom_to_string(root: &Option<Rc<RefCell<Node>>>) -> String {
    let mut result = String::from("\n");
    convert_dom_to_string_inner(root, 0, &mut result);
    result
}

fn convert_dom_to_string_inner(
    node: &Option<Rc<RefCell<Node>>>,
    depth: usize,
    result: &mut String,
) {
    if let Some(node) = node {
        result.push_str(&"  ".repeat(depth));
        result.push_str(&format!("{:?}", node.borrow().kind()));
        result.push('\n');
        convert_dom_to_string_inner(&node.borrow().first_child(), depth + 1, result);
        convert_dom_to_string_inner(&node.borrow().next_sibling(), depth, result);
    }
}

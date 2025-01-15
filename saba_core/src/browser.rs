use core::cell::RefCell;

use alloc::{
    rc::{Rc, Weak},
    string::String,
    vec::Vec,
};

use crate::{
    http::HttpResponse,
    renderer::{
        dom::node::Window,
        html::{parser::HtmlParser, token::HtmlTokenizer},
    },
    utils::convert_dom_to_string,
};

#[derive(Debug, Clone)]
pub struct Browser {
    active_page_index: usize,
    pages: Vec<Rc<RefCell<Page>>>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut page = Page::new();

        let browser = Rc::new(RefCell::new(Self {
            active_page_index: 0,
            pages: Vec::new(),
        }));

        page.set_browser(Rc::downgrade(&browser));
        browser.borrow_mut().pages.push(Rc::new(RefCell::new(page)));
        browser
    }

    pub fn current_page(&self) -> Rc<RefCell<Page>> {
        self.pages[self.active_page_index].clone()
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    browser: Weak<RefCell<Browser>>,
    frame: Option<Rc<RefCell<Window>>>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            browser: Weak::new(),
            frame: None,
        }
    }

    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }

    pub fn receive_response(&mut self, response: HttpResponse) -> String {
        self.create_frame(response.body());

        // デバッグ用に DOM ツリーを文字列として返す
        if let Some(frame) = &self.frame {
            let dom = frame.borrow().document().clone();
            convert_dom_to_string(&Some(dom))
        } else {
            String::new()
        }
    }

    fn create_frame(&mut self, html: String) {
        let html_tokenizer = HtmlTokenizer::new(html);
        let frame = HtmlParser::new(html_tokenizer).construct_tree();
        self.frame = Some(frame);
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

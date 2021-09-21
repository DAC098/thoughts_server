use handlebars::{Handlebars, RenderError};
use serde::{Serialize};

pub struct TemplateState<'a> {
    hb: Handlebars<'a>
}

impl<'a> TemplateState<'a> {

    pub fn new(hb: Handlebars<'a>) -> TemplateState<'a> {
        TemplateState { hb }
    }

    pub fn render<T>(
        &self, name: &str, data: &T
    ) -> std::result::Result<String, RenderError>
    where
        T: Serialize
    {
        self.hb.render(name, data)
    }
}
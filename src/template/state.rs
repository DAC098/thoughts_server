use actix_web::web;
use handlebars::{Handlebars, RenderError};
use serde::Serialize;

pub struct TemplateState<'a> {
    hb: Handlebars<'a>
}

pub type WebTemplateState<'a> = web::Data<TemplateState<'a>>;

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

    pub fn render_email_parts<T>(
        &self,
        name: &str,
        data: &T
    ) -> std::result::Result<(String, String), RenderError>
    where
        T: Serialize
    {
        let mut text_name = String::with_capacity(name.len() + 6 + 5);
        text_name.push_str("email/");
        text_name.push_str(name);
        text_name.push_str(".text");
        let mut html_name = String::with_capacity(name.len() + 6 + 5);
        html_name.push_str("email/");
        html_name.push_str(name);
        html_name.push_str(".html");

        let text_body = self.hb.render(&text_name, data)?;
        let html_body = self.hb.render(&html_name, data)?;

        Ok((text_body, html_body))
    }
}
use serde_json::{to_string};
use handlebars::{
    Handlebars,
    HelperDef,
    RenderContext,
    Helper,
    Context,
    HelperResult,
    Output,
    RenderError
};

#[derive(Clone, Copy)]
pub struct JsJson;

impl HelperDef for JsJson {
    fn call<'reg: 'rc, 'rc>(
        &self, 
        h: &Helper, 
        _hb: &Handlebars, 
        _context: &Context, 
        _rc: &mut RenderContext, 
        out: &mut dyn Output
    ) -> HelperResult {
        let param = h.param(0).ok_or(RenderError::new("missing parameter to transform"))?;

        if let Ok(value_str) = to_string(param.value()) {
            out.write(value_str.as_str())?;
            Ok(())
        } else {
            Err(RenderError::new("failed to convert given value to json string"))
        }
    }
}


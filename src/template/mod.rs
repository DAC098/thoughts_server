use std::{fs};
use std::fmt::{Write};
use std::path::{PathBuf};
use std::ffi::{OsString};

use handlebars::{Handlebars};
use lazy_static::lazy_static;

use tlib::config::{TemplateConfig};

pub mod helpers;

use crate::error;

lazy_static! {
    static ref HBS_EXTENSION: OsString = OsString::from("hbs");
}

fn recursive_load_directory<'a>(
    config: &TemplateConfig,
    hb: &mut Handlebars<'a>,
    directory: &PathBuf,
    template_errors: &mut Vec<handlebars::TemplateError>,
) -> error::Result<()> {
    for item in fs::read_dir(directory)? {
        let file = item?;
        let path = file.path();

        if path.is_dir() {
            recursive_load_directory(config, hb, &path, template_errors)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == HBS_EXTENSION.as_os_str() {
                    let name = path.strip_prefix(&config.directory).unwrap();

                    if let Some(name_str) = name.to_str() {
                        if let Err(err) = hb.register_template_file(
                            name_str.strip_suffix(".hbs").unwrap(),
                            &path
                        ) {
                            template_errors.push(err);
                        }
                    } else {
                        log::info!("path contains invalid unicode characters: {}", name.display());
                    }
                }
            } else {
                log::info!("failed to determine extension for file: {}", path.display());
            }
        }
    }

    Ok(())
}

fn register_helpers<'a>(
    _config: &TemplateConfig,
    hb: &mut Handlebars<'a>,
) -> error::Result<()> {
    hb.register_helper("js-json", Box::new(helpers::JsJson));

    Ok(())
}

pub fn get_built_registry<'a>(config: TemplateConfig) -> error::Result<Handlebars<'a>> {
    let required_templates = [
        "pages/index"
    ];

    let mut hb = Handlebars::new();
    hb.set_dev_mode(config.dev_mode);

    register_helpers(&config, &mut hb)?;

    let mut template_errors: Vec<handlebars::TemplateError> = Vec::new();

    recursive_load_directory(&config, &mut hb, &config.directory, &mut template_errors)?;

    if template_errors.len() > 0 {
        let mut msg = "there were errors when attempting to load templates:\n".to_owned();

        for err in template_errors {
            // this should probably be changed to actually handle the error
            msg.write_fmt(format_args!("{}\n", err)).unwrap();
        }

        return Err(error::AppError::General(msg));
    }

    let mut missing_templates = "the following templates are missing from the registry:\n".to_owned();
    let mut found_missing = false;

    for name in required_templates {
        if !hb.has_template(name) {
            // this should probably be changed to actually handle the error
            missing_templates.write_fmt(format_args!("{}\n", name)).unwrap();
            found_missing = true;
        }
    }

    if found_missing {
        return Err(error::AppError::General(missing_templates));
    }

    Ok(hb)
}


use std::fs::{self, ReadDir};
use std::fmt::Write;
use std::path::PathBuf;
use std::ffi::OsStr;

use handlebars::Handlebars;

use crate::config::TemplateConfig;
use crate::error;

pub mod helpers;
pub mod state;
pub use state::*;

struct WorkingItem {
    iter: ReadDir,
    depth: usize
}

fn load_directory<'a>(
    conf: &TemplateConfig,
    hb: &mut Handlebars<'a>,
    directory: &PathBuf,
    template_errors: &mut Vec<handlebars::TemplateError>,
) -> error::Result<()> {
    let hbs_extesion = OsStr::new("hbs");
    let mut working_queue = Vec::with_capacity(3);
    working_queue.push(WorkingItem {
        iter: fs::read_dir(directory)?,
        depth: 0
    });

    while let Some(mut working) = working_queue.pop() {
        while let Some(entry) = working.iter.next() {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                if working.depth < 10 {
                    if working_queue.len() + 1 == working_queue.capacity() {
                        working_queue.reserve(3);
                    }

                    let depth = working.depth + 1;

                    working_queue.push(working);
                    working_queue.push(WorkingItem {
                        iter: fs::read_dir(&entry_path)?,
                        depth
                    });

                    break;
                } else {
                    log::info!("max load directory depth reached");
                }
            } else if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if ext == hbs_extesion {
                        let name = entry_path.strip_prefix(&conf.directory).unwrap();

                        if let Some(name_str) = name.to_str() {
                            if let Err(err) = hb.register_template_file(
                                name_str.strip_suffix(".hbs").unwrap(),
                                &entry_path
                            ) {
                                template_errors.push(err);
                            }
                        } else {
                            log::info!("path contains invalid unicode characters: {}", name.display());
                        }
                    }
                } else {
                    log::info!("failed to determine extension for file: {}", entry_path.display());
                }
            }
        }
    }

    Ok(())
}

pub fn get_built_registry<'a>(config: TemplateConfig) -> error::Result<Handlebars<'a>> {
    let required_templates = [
        "pages/index",
        "email/verify_email.text",
        "email/verify_email.html"
    ];

    let mut hb = Handlebars::new();
    hb.set_dev_mode(config.dev_mode);
    hb.register_helper("js-json", Box::new(helpers::JsJson));

    let mut template_errors: Vec<handlebars::TemplateError> = Vec::new();

    load_directory(&config, &mut hb, &config.directory, &mut template_errors)?;

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

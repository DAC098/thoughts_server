use actix_web::{web};

pub mod db;
pub mod template;
pub mod email;
pub mod server_info;

pub type WebDbState = web::Data<db::DBState>;
pub type WebTemplateState<'a> = web::Data<template::TemplateState<'a>>;
pub type WebEmailState = web::Data<email::EmailState>;
pub type WebSserverInfoState = web::Data<server_info::ServerInfoState>;
use actix_web::{web};

pub mod db;
pub mod template;
pub mod email;
pub mod server_info;
pub mod storage;

pub type WebDbState = web::Data<db::DBState>;
pub type WebTemplateState<'a> = web::Data<template::TemplateState<'a>>;
pub type WebEmailState = web::Data<email::EmailState>;
pub type WebServerInfoState = web::Data<server_info::ServerInfoState>;
pub type WebStorageState = web::Data<storage::StorageState>;
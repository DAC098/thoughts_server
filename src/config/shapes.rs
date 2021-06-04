use serde::{Deserialize};

pub trait MapShape {
    fn map_shape(&mut self, rhs: Self);
}

#[inline]
fn assign_map_value<T>(lhs: &mut Option<T>, rhs: Option<T>) {
    if rhs.is_some() { *lhs = rhs; }
}

#[inline]
fn assign_map_struct<T>(lhs: &mut Option<T>, rhs: Option<T>) 
where
    T: MapShape
{
    if let Some(lhs_value) = lhs.as_mut() {
        if let Some(rhs_value) = rhs {
            lhs_value.map_shape(rhs_value);
        }
    } else {
        *lhs = rhs;
    }
}

#[derive(Deserialize)]
pub struct DBConfigShape {
    pub username: Option<String>,
    pub password: Option<String>,

    pub database: Option<String>,

    pub hostname: Option<String>,
    pub port: Option<u16>
}

impl MapShape for DBConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.username, rhs.username);
        assign_map_value(&mut self.password, rhs.password);
        assign_map_value(&mut self.database, rhs.database);
        assign_map_value(&mut self.hostname, rhs.hostname);
        assign_map_value(&mut self.port, rhs.port);
    }
}

#[derive(Deserialize)]
pub struct BindInterfaceShape {
    pub host: String,
    pub port: Option<u16>
}

#[derive(Deserialize)]
pub struct SessionConfigShape {
    pub domain: Option<String>
}

impl MapShape for SessionConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.domain, rhs.domain);
    }
}

#[derive(Deserialize)]
pub struct EmailConfigShape {
    pub enable: Option<bool>,
    pub from: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub relay: Option<String>
}

impl MapShape for EmailConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.enable, rhs.enable);
        assign_map_value(&mut self.from, rhs.from);
        assign_map_value(&mut self.username, rhs.username);
        assign_map_value(&mut self.password, rhs.password);
        assign_map_value(&mut self.relay, rhs.relay);
    }
}

#[derive(Deserialize)]
pub struct ServerInfoConfigShape {
    pub secure: Option<bool>,
    pub origin: Option<String>,
    pub name: Option<String>
}

impl MapShape for ServerInfoConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.secure, rhs.secure);
        assign_map_value(&mut self.origin, rhs.origin);
        assign_map_value(&mut self.name, rhs.name);
    }
}

#[derive(Deserialize)]
pub struct SslConfigShape {
    pub enable: Option<bool>,
    pub key: Option<String>,
    pub cert: Option<String>
}

impl MapShape for SslConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.enable, rhs.enable);
        assign_map_value(&mut self.key, rhs.key);
        assign_map_value(&mut self.cert, rhs.cert);
    }
}

#[derive(Deserialize)]
pub struct ServerConfigShape {
    pub bind: Option<Vec<BindInterfaceShape>>,
    pub port: Option<u16>,

    pub threads: Option<usize>,
    pub backlog: Option<u32>,
    pub max_connections: Option<usize>,
    pub max_connection_rate: Option<usize>,

    pub db: Option<DBConfigShape>,
    pub session: Option<SessionConfigShape>,
    pub email: Option<EmailConfigShape>,
    pub info: Option<ServerInfoConfigShape>,
    pub ssl: Option<SslConfigShape>
}

impl MapShape for ServerConfigShape {
    fn map_shape(&mut self, rhs: Self) {
        assign_map_value(&mut self.bind, rhs.bind);
        assign_map_value(&mut self.port, rhs.port);
        assign_map_value(&mut self.threads, rhs.threads);
        assign_map_value(&mut self.backlog, rhs.backlog);
        assign_map_value(&mut self.max_connections, rhs.max_connections);
        assign_map_value(&mut self.max_connection_rate, rhs.max_connection_rate);
    
        assign_map_struct(&mut self.session, rhs.session);
        assign_map_struct(&mut self.db, rhs.db);
        assign_map_struct(&mut self.email, rhs.email);
        assign_map_struct(&mut self.info, rhs.info);
        assign_map_struct(&mut self.ssl, rhs.ssl);
    }
}
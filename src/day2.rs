use std::{
    error::Error, net::{Ipv4Addr, Ipv6Addr}, ops::BitXor, str::FromStr
};

use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;

#[derive(Deserialize)]
struct Ipv4P1 {
    from: String,
    key: String,
}
impl Ipv4P1 {
    fn cal(self) ->Result<String,Box<dyn Error>> {
        let f = Ipv4Addr::from_str(&self.from)?;
        let k = Ipv4Addr::from_str(&self.key)?;
        let v = f
            .octets()
            .into_iter()
            .zip(k.octets().into_iter())
            .map(|(l, r)| l.overflowing_add(r).0)
            .collect::<Vec<_>>();

        let addr = unsafe {
            let p = v.as_ptr() as *const [u8; 4];
            Ipv4Addr::from(*p).to_string()
        };
        Ok(addr)
    }
}

#[derive(Deserialize)]
struct Ipv4P2 {
    from: String,
    to: String,
}
impl Ipv4P2 {
    fn cal(self) ->Result<String,Box<dyn Error>> {
        let t = Ipv4Addr::from_str(&self.to)?;
        let f = Ipv4Addr::from_str(&self.from)?;
        let v = t
            .octets()
            .into_iter()
            .zip(f.octets().into_iter())
            .map(|(l, r)| l.overflowing_sub(r).0)
            .collect::<Vec<_>>();

        let addr = unsafe {
            let p = v.as_ptr() as *const [u8; 4];
            Ipv4Addr::from(*p).to_string()
        };
        Ok(addr)
    }
}

#[derive(Deserialize)]
struct Ipv6P1 {
    from: String,
    key: String,
}
impl Ipv6P1 {
    fn cal(self) ->Result<String,Box<dyn Error>> {
        let f = Ipv6Addr::from_str(&self.from)?;

        let k = Ipv6Addr::from_str(&self.key)?;
        let v: Vec<_> = f
            .octets()
            .into_iter()
            .zip(k.octets().into_iter())
            .map(|(l, r)| l.bitxor(r))
            .collect();

        let addr = unsafe {
            let p = v.as_ptr() as *const [u8; 16];
            Ipv6Addr::from(*p).to_string()
        };
        Ok(addr)
    }
}
#[derive(Deserialize)]
struct Ipv6P2 {
    from: String,
    to: String,
}
impl Ipv6P2 {
    fn cal(self) ->Result<String,Box<dyn Error>> {
        let t = Ipv6Addr::from_str(&self.to)?;
        let f = Ipv6Addr::from_str(&self.from)?;
        let v = t
            .octets()
            .into_iter()
            .zip(f.octets().into_iter())
            .map(|(l, r)| l.bitxor(r))
            .collect::<Vec<_>>();

        let addr = unsafe {
            let p = v.as_ptr() as *const [u8; 16];
            Ipv6Addr::from(*p).to_string()
        };
        Ok(addr)
    }
}
async fn v4p1(Query(payload): Query<Ipv4P1>) -> String {
    payload.cal().unwrap()
}

async fn v4p2(Query(payload): Query<Ipv4P2>) -> String {
    payload.cal().unwrap()
}
async fn v6p1(Query(payload): Query<Ipv6P1>) -> String {
    payload.cal().unwrap()
}
async fn v6p2(Query(payload): Query<Ipv6P2>) -> String {
    payload.cal().unwrap()
}

pub fn router() -> Router {
    Router::new()
        .route("/2/dest", get(v4p1))
        .route("/2/key", get(v4p2))
        .route("/2/v6/dest", get(v6p1))
        .route("/2/v6/key", get(v6p2))
}

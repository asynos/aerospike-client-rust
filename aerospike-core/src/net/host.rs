// Copyright 2015-2018 Aerospike, Inc.
//
// Portions may be licensed to Aerospike, Inc. under one or more contributor
// license agreements.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use std::fmt;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::vec::IntoIter;

use crate::errors::{ErrorKind, Result, ResultExt};
use crate::net::parser::Parser;

/// Host name/port of database server.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Host {
    /// Host name or IP address of database server.
    pub name: String,

    /// Port of database server.
    pub port: u16,

    /// Name used for TLS connection
    pub tls_name: Option<String>,
}

impl Host {
    /// Create a new host instance given a hostname/IP and a port number.
    pub fn new(name: &str, port: u16, tls_name: Option<&str>) -> Self {
        Host {
            name: name.to_string(),
            port,
            tls_name: tls_name.map(|x| x.to_owned()),
        }
    }

    /// Returns a string representation of the host's address.
    pub fn address(&self) -> String {
        format!("{}:{}", self.name, self.port)
    }

    /// Returns a string representation of domain name for TLS conenction.
    pub fn tls_name(&self) -> Option<&str> {
        self.tls_name.as_ref().map(|x| x.as_str())
    }
}

impl ToSocketAddrs for Host {
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<IntoIter<SocketAddr>> {
        (self.name.as_str(), self.port).to_socket_addrs()
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.tls_name {
            Some(tls_name) => write!(f, "{}:{}:{}", self.name, tls_name, self.port),
            None => write!(f, "{}:{}", self.name, self.port),
        }
    }
}

/// A trait for objects which can be converted to one or more `Host` values.
pub trait ToHosts {
    /// Converts this object into a list of `Host`s.
    ///
    /// # Errors
    ///
    /// Any errors encountered during conversion will be returned as an `Err`.
    fn to_hosts(&self) -> Result<Vec<Host>>;
}

impl ToHosts for Vec<Host> {
    fn to_hosts(&self) -> Result<Vec<Host>> {
        Ok(self.clone())
    }
}

impl ToHosts for String {
    fn to_hosts(&self) -> Result<Vec<Host>> {
        let mut parser = Parser::new(self, 3000);
        parser
            .read_hosts()
            .chain_err(|| ErrorKind::InvalidArgument(format!("Invalid hosts list: '{}'", self)))
    }
}

impl<'a> ToHosts for &'a str {
    fn to_hosts(&self) -> Result<Vec<Host>> {
        (*self).to_string().to_hosts()
    }
}

#[cfg(test)]
mod tests {
    use super::{Host, ToHosts};

    #[test]
    fn to_hosts() {
        assert_eq!(
            vec![Host::new("foo", 3000, None)],
            String::from("foo").to_hosts().unwrap()
        );
        assert_eq!(vec![Host::new("foo", 3000, None)], "foo".to_hosts().unwrap());
        assert_eq!(vec![Host::new("foo", 1234, None)], "foo:1234".to_hosts().unwrap());
        assert_eq!(
            vec![Host::new("foo", 1234, None), Host::new("bar", 1234, None)],
            "foo:1234,bar:1234".to_hosts().unwrap()
        );
        assert_eq!(
            vec![Host::new("foo", 1234, Some("bar")), Host::new("bar", 1234, Some("foo"))],
            "foo:bar:1234,bar:foo:1234".to_hosts().unwrap()
        );
    }
}

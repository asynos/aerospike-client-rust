// Copyright 2015-2016 Aerospike, Inc.
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

extern crate core;

use std::fmt;


use error::{AerospikeResult, ResultCode, AerospikeError};
use value::{Value, IntValue, StringValue};
use value;
use common::ParticleType;

use crypto::ripemd160::Ripemd160;
use crypto::digest::Digest;

// #[derive(Debug)]
pub struct Key<'a> {
    pub namespace: &'a str,
    pub set_name: &'a str,
    pub digest: Vec<u8>,
    pub user_key: &'a Value,
}

impl<'a> Key<'a> {
    pub fn new(namespace: &'a str, setname: &'a str, key: &'a Value) -> AerospikeResult<Self> {
        let mut key = Key {
            namespace: namespace,
            set_name: setname,
            digest: Vec::with_capacity(20),
            user_key: key,
        };

        key.digest.resize(20, 0);
        try!(key.compute_digest());
        Ok(key)
    }

    fn compute_digest(&mut self) -> AerospikeResult<()> {
        match self.user_key.particle_type() {
            ParticleType::NULL => {
                return Err(AerospikeError::new(ResultCode::PARAMETER_ERROR,
                                               Some("Invalid Key `null`".to_string())))
            }
            ParticleType::MAP => {
                return Err(AerospikeError::new(ResultCode::PARAMETER_ERROR,
                                               Some("Invalid Key: maps are not allowed."
                                                        .to_string())))
            }
            _ => (),
        }

        let mut hash = Ripemd160::new();
        hash.input(self.set_name.as_bytes());
        hash.input(&[self.user_key.particle_type() as u8]);
        hash.input(&try!(self.user_key.as_bytes()));
        hash.result(&mut self.digest);

        Ok(())
    }
}

impl<'a> core::fmt::Display for Key<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        try!(self.namespace.fmt(f));
        Ok(())
    }
}

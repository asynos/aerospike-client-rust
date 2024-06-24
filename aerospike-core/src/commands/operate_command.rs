// Copyright 2015-2018 Aerospike, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;
use std::time::Duration;

use crate::cluster::{Cluster, Node};
use crate::commands::{Command, SingleCommand};
use crate::errors::Result;
use crate::net::Connection;
use crate::operations::Operation;
use crate::policy::WritePolicy;
use crate::value::bytes_to_particle;
use crate::{Key, ResultCode, Value};

pub struct OperateCommand<'a> {
    pub single_command: SingleCommand<'a>,
    pub record: OperateRecord,
    policy: &'a WritePolicy,
    operations: &'a [Operation<'a>],
}

/// The return value from operate. Like a record, but retains the order and duplicate keys of bins.
#[derive(Default)]
pub struct OperateRecord {
    /// Record key. When reading a record from the database, the key is not set in the returned
    /// Record struct.
    pub key: Option<Key>,

    /// Map of named record bins.
    pub bins: Vec<(String, Value)>,

    /// Record modification count.
    pub generation: u32,

    /// Date record will expire, in seconds from Jan 01 2010, 00:00:00 UTC.
    expiration: u32,
}

impl<'a> OperateCommand<'a> {
    pub fn new(
        policy: &'a WritePolicy,
        cluster: Arc<Cluster>,
        key: &'a Key,
        operations: &'a [Operation<'a>],
    ) -> Self {
        OperateCommand {
            single_command: SingleCommand::new(cluster, key, crate::policy::Replica::Master),
            record: OperateRecord::default(),
            policy,
            operations,
        }
    }

    pub async fn execute(&mut self) -> Result<()> {
        SingleCommand::execute(self.policy, self).await
    }

    fn parse_record(
        &mut self,
        conn: &mut Connection,
        op_count: usize,
        field_count: usize,
    ) -> Result<()> {
        // There can be fields in the response (setname etc). For now, ignore them. Expose them to
        // the API if needed in the future.
        for _ in 0..field_count {
            let field_size = conn.buffer.read_u32(None) as usize;
            conn.buffer.skip(4 + field_size);
        }

        self.record.bins.reserve_exact(op_count);
        for _ in 0..op_count {
            let op_size = conn.buffer.read_u32(None) as usize;
            conn.buffer.skip(1);
            let particle_type = conn.buffer.read_u8(None);
            conn.buffer.skip(1);
            let name_size = conn.buffer.read_u8(None) as usize;
            let name: String = conn.buffer.read_str(name_size)?;

            let particle_bytes_size = op_size - (4 + name_size);
            let value = bytes_to_particle(particle_type, &mut conn.buffer, particle_bytes_size)?;
            self.record.bins.push((name, value));
        }

        Ok(())
    }

}

#[async_trait::async_trait]
impl<'a> Command for OperateCommand<'a> {
    async fn write_timeout(
        &mut self,
        conn: &mut Connection,
        timeout: Option<Duration>,
    ) -> Result<()> {
        conn.buffer.write_timeout(timeout);
        Ok(())
    }

    async fn write_buffer(&mut self, conn: &mut Connection) -> Result<()> {
        conn.flush().await
    }

    fn prepare_buffer(&mut self, conn: &mut Connection) -> Result<()> {
        conn.buffer.set_operate(
            self.policy,
            self.single_command.key,
            self.operations,
        )
    }

    fn get_node(&mut self) -> Result<Arc<Node>> {
        self.single_command.get_node()
    }

    async fn parse_result(&mut self, conn: &mut Connection) -> Result<()> {
        if let Err(err) = conn
            .read_buffer(super::buffer::MSG_TOTAL_HEADER_SIZE as usize)
            .await
        {
            warn!("Parse result error: {}", err);
            bail!(err);
        }

        conn.buffer.reset_offset();
        let sz = conn.buffer.read_u64(Some(0));
        let header_length = conn.buffer.read_u8(Some(8));
        let result_code = conn.buffer.read_u8(Some(13));
        self.record.generation = conn.buffer.read_u32(Some(14));
        self.record.expiration = conn.buffer.read_u32(Some(18));
        let field_count = conn.buffer.read_u16(Some(26)) as usize; // almost certainly 0
        let op_count = conn.buffer.read_u16(Some(28)) as usize;
        let receive_size = ((sz & 0xFFFF_FFFF_FFFF) - u64::from(header_length)) as usize;

        // Read remaining message bytes
        if receive_size > 0 {
            if let Err(err) = conn.read_buffer(receive_size).await {
                warn!("Parse result error: {}", err);
                bail!(err);
            }
        }

        match ResultCode::from(result_code) {
            ResultCode::Ok => {
                self.parse_record(conn, op_count, field_count)
            }
            ResultCode::UdfBadResponse => {
                // record bin "FAILURE" contains details about the UDF error
                self.parse_record(conn, op_count, field_count)?;
                let reason = self.record
                    .bins.iter().find(|(k, _)|k == "FAILURE").map(|(_, v)|v)
                    .map_or(String::from("UDF Error"), ToString::to_string);
                Err(crate::ErrorKind::UdfBadResponse(reason).into())
            }
            rc => Err(crate::ErrorKind::ServerError(rc).into()),
        }
    }
}

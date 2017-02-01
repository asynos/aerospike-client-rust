// Copyright 2015-2017 Aerospike, Inc.
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

pub mod scalar;
pub mod lists;
pub mod maps;

use std::collections::HashMap;

pub use self::maps::{MapOrder, MapReturnType, MapWriteMode, MapPolicy};
pub use self::scalar::*;

use errors::*;
use Value;
use msgpack::encoder;
use commands::ParticleType;
use commands::buffer::Buffer;

#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub enum OperationType {
    Read = 1,
    Write = 2,
    CdtRead = 3,
    CdtWrite = 4,
    Incr = 5,
    Append = 9,
    Prepend = 10,
    Touch = 11,
}

#[derive(Debug)]
#[doc(hidden)]
pub enum OperationData<'a> {
    None,
    Value(&'a Value),
    CdtListOp(CdtOperation<'a>),
    CdtMapOp(CdtOperation<'a>),
}

#[derive(Debug)]
#[doc(hidden)]
pub enum OperationBin<'a> {
    None,
    All,
    Name(&'a str),
}

#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub enum CdtOpType {
    ListAppend = 1,
    ListAppendItems = 2,
    ListInsert = 3,
    ListInsertItems = 4,
    ListPop = 5,
    ListPopRange = 6,
    ListRemove = 7,
    ListRemoveRange = 8,
    ListSet = 9,
    ListTrim = 10,
    ListClear = 11,
    ListSize = 16,
    ListGet = 17,
    ListGetRange = 18,
    MapSetType = 64,
    MapAdd = 65,
    MapAddItems = 66,
    MapPut = 67,
    MapPutItems = 68,
    MapReplace = 69,
    MapReplaceItems = 70,
    MapIncrement = 73,
    MapDecrement = 74,
    MapClear = 75,
    MapRemoveByKey = 76,
    MapRemoveByIndex = 77,
    MapRemoveByValue = 78,
    MapRemoveByRank = 79,
    MapRemoveByKeyList = 81,
    MapRemoveByValueList = 83,
    MapRemoveByKeyInterval = 84,
    MapRemoveByIndexRange = 85,
    MapRemoveByValueInterval = 86,
    MapRemoveByRankRange = 87,
    MapSize = 96,
    MapGetByKey = 97,
    MapGetByIndex = 98,
    MapGetByValue = 99,
    MapGetByRank = 100,
    MapGetByKeyInterval = 103,
    MapGetByIndexRange = 104,
    MapGetByValueInterval = 105,
    MapGetByRankRange = 106,
}

#[derive(Debug)]
#[doc(hidden)]
pub enum CdtArgument<'a> {
    Byte(u8),
    Int(i64),
    Value(&'a Value),
    List(&'a [Value]),
    Map(&'a HashMap<Value, Value>),
}

#[derive(Debug)]
#[doc(hidden)]
pub struct CdtOperation<'a> {
    pub op: CdtOpType,
    pub args: Vec<CdtArgument<'a>>,
}

impl<'a> CdtOperation<'a> {
    fn particle_type(&self) -> ParticleType {
        ParticleType::BLOB
    }

    fn estimate_size(&self) -> Result<usize> {
        let size: usize = try!(encoder::pack_cdt_op(&mut None, self));
        Ok(size)
    }

    fn write_to(&self, buffer: &mut Buffer) -> Result<usize> {
        let size: usize = try!(encoder::pack_cdt_op(&mut Some(buffer), self));
        Ok(size)
    }
}

/// Database operation definition. This data type is used in the client's `operate()` method.
#[derive(Debug)]
pub struct Operation<'a> {
    // OpType determines type of operation.
    #[doc(hidden)] pub op: OperationType,

    // BinName (Optional) determines the name of bin used in operation.
    #[doc(hidden)] pub bin: OperationBin<'a>,

    // BinData determines bin value used in operation.
    #[doc(hidden)] pub data: OperationData<'a>,
}

impl<'a> Operation<'a> {
    pub fn estimate_size(&self) -> Result<usize> {
        let mut size: usize = 0;
        size += match self.bin {
            OperationBin::Name(bin) => bin.len(),
            OperationBin::None | OperationBin::All => 0
        };
        size += match self.data {
            OperationData::None => 0,
            OperationData::Value(ref value) => try!(value.estimate_size()),
            OperationData::CdtListOp(ref cdt_op) |
                OperationData::CdtMapOp(ref cdt_op) => try!(cdt_op.estimate_size()),
        };

        Ok(size)
    }

    pub fn write_to(&self, buffer: &mut Buffer) -> Result<usize> {
        let mut size: usize = 0;

        // remove the header size from the estimate
        let op_size = try!(self.estimate_size());

        size += try!(buffer.write_u32(op_size as u32 + 4));
        size += try!(buffer.write_u8(self.op as u8));

        match self.data {
            OperationData::None => {
                size += try!(self.write_op_header_to(buffer, ParticleType::NULL as u8));
            }
            OperationData::Value(ref value) => {
                size += try!(self.write_op_header_to(buffer, value.particle_type() as u8));
                size += try!(value.write_to(buffer));
            }
            OperationData::CdtListOp(ref cdt_op) | OperationData::CdtMapOp(ref cdt_op) => {
                size += try!(self.write_op_header_to(buffer, cdt_op.particle_type() as u8));
                size += try!(cdt_op.write_to(buffer));
            }
        };

        Ok(size)
    }

    fn write_op_header_to(&self, buffer: &mut Buffer, particle_type: u8) -> Result<usize> {
        let mut size = try!(buffer.write_u8(particle_type as u8));
        size += try!(buffer.write_u8(0));
        match self.bin {
            OperationBin::Name(bin) => {
                size += try!(buffer.write_u8(bin.len() as u8));
                size += try!(buffer.write_str(bin));
            }
            OperationBin::None | OperationBin::All => {
                size += try!(buffer.write_u8(0));
            }
        }
        Ok(size)
    }
}

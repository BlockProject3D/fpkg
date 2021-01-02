// Copyright (c) 2020, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::convert::TryInto;

pub trait GenericArrayLen
{
    const size: usize;
    type TArray;

    fn from_array(buf: &[u8]) -> Self::TArray;
}

pub struct T2
{
}

impl GenericArrayLen for T2
{
    type TArray = [u8; 2];
    const size: usize = 2;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T3
{
}

impl GenericArrayLen for T3
{
    type TArray = [u8; 3];
    const size: usize = 3;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T4
{
}

impl GenericArrayLen for T4
{
    type TArray = [u8; 4];
    const size: usize = 4;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T5
{
}

impl GenericArrayLen for T5
{
    type TArray = [u8; 5];
    const size: usize = 5;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T6
{
}

impl GenericArrayLen for T6
{
    type TArray = [u8; 6];
    const size: usize = 6;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T7
{
}

impl GenericArrayLen for T7
{
    type TArray = [u8; 7];
    const size: usize = 7;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T8
{
}

impl GenericArrayLen for T8
{
    type TArray = [u8; 8];
    const size: usize = 8;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T9
{
}

impl GenericArrayLen for T9
{
    type TArray = [u8; 9];
    const size: usize = 9;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T10
{
}

impl GenericArrayLen for T10
{
    type TArray = [u8; 10];
    const size: usize = 10;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T11
{
}

impl GenericArrayLen for T11
{
    type TArray = [u8; 11];
    const size: usize = 11;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T12
{
}

impl GenericArrayLen for T12
{
    type TArray = [u8; 12];
    const size: usize = 12;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T13
{
}

impl GenericArrayLen for T13
{
    type TArray = [u8; 13];
    const size: usize = 13;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T14
{
}

impl GenericArrayLen for T14
{
    type TArray = [u8; 14];
    const size: usize = 14;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T15
{
}

impl GenericArrayLen for T15
{
    type TArray = [u8; 15];
    const size: usize = 15;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub struct T16
{
}

impl GenericArrayLen for T16
{
    type TArray = [u8; 16];
    const size: usize = 16;

    fn from_array(buf: &[u8]) -> Self::TArray
    {
        return buf.try_into().unwrap();
    }
}

pub fn extract_slice<TArray: GenericArrayLen>(large_buf: &[u8], offset: usize) -> TArray::TArray
{
    let buf = &large_buf[offset..TArray::size];
    return TArray::from_array(buf);
}

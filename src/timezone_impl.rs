// Copyright (c) 2022, Yuri6037
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
// * Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
// * Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
// * Neither the name of time-tz nor the names of its contributors
// may be used to endorse or promote products derived from this software
// without specific prior written permission.
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

use std::cmp::Ordering;
use std::ops::Index;
use crate::binary_search::binary_search;
use time::{OffsetDateTime, UtcOffset};
use crate::TimeZone;

//Inspired from https://github.com/chronotope/chrono-tz/blob/main/src/timezone_impl.rs

struct Span {
    start: Option<i64>,
    end: Option<i64>
}

impl Span {
    fn cmp(&self, x: i64) -> Ordering {
        match (self.start, self.end) {
            (Some(a), Some(b)) if a <= x && x < b => Ordering::Equal,
            (Some(a), Some(b)) if a <= x && b <= x => Ordering::Less,
            (Some(_), Some(_)) => Ordering::Greater,
            (Some(a), None) if a <= x => Ordering::Equal,
            (Some(_), None) => Ordering::Greater,
            (None, Some(b)) if b <= x => Ordering::Less,
            (None, Some(_)) => Ordering::Equal,
            (None, None) => Ordering::Equal,
        }
    }
}

pub struct FixedTimespan {
    pub utc_offset: i64,
    pub dst_offset: i64,
    pub name: &'static str,
}

pub struct FixedTimespanSet
{
    pub first: FixedTimespan,
    pub others: &'static [(i64, FixedTimespan)]
}

impl FixedTimespanSet {
    fn len(&self) -> usize {
        1 + self.others.len()
    }

    fn span_utc(&self, i: usize) -> Span {
        let start = match i {
            0 => None,
            _ => Some(self.others[i - 1].0)
        };
        let end;
        if i >= self.others.len() {
            end = None;
        } else {
            end = Some(self.others[i].0)
        }
        Span { start, end }
    }
}

impl Index<usize> for FixedTimespanSet {
    type Output = FixedTimespan;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len());
        match index {
            0 => &self.first,
            _ => &self.others[index - 1].1
        }
    }
}

pub struct Tz {
    set: &'static FixedTimespanSet
}

impl TimeZone for Tz {
    fn get_offset_utc(&self, date_time: &OffsetDateTime) -> UtcOffset {
        let timestamp = date_time.unix_timestamp();
        let index = binary_search(0, self.set.len(),
                                  |i| self.set.span_utc(i).cmp(timestamp)).unwrap();
        let secs = &self.set[index].utc_offset;
        UtcOffset::from_whole_seconds(*secs as i32).unwrap()
    }
}

pub const fn internal_tz_new(set: &'static FixedTimespanSet) -> Tz {
    Tz { set }
}

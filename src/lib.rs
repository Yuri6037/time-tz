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

use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

pub trait TimeZone
{
    fn get_offset_utc(&self, date_time: &OffsetDateTime) -> UtcOffset;
}

pub trait OffsetDateTimeExt
{
    fn to_timezone<T: TimeZone>(&self, tz: &T) -> OffsetDateTime;
}

pub trait PrimitiveDateTimeExt
{
    fn assume_timezone<T: TimeZone>(&self, tz: &T) -> OffsetDateTime;
}

impl PrimitiveDateTimeExt for PrimitiveDateTime {
    fn assume_timezone<T: TimeZone>(&self, tz: &T) -> OffsetDateTime {
        let offset = tz.get_offset_utc(&self.assume_utc());
        self.assume_offset(offset)
    }
}

impl OffsetDateTimeExt for OffsetDateTime {
    fn to_timezone<T: TimeZone>(&self, tz: &T) -> OffsetDateTime {
        let offset = tz.get_offset_utc(self);
        self.to_offset(offset)
    }
}

mod timezone_impl;
pub mod timezones;
mod binary_search;

pub use timezone_impl::Tz;
//pub use timezones::get as get_timezone_by_name;
pub use timezones::root as timezone;

#[cfg(test)]
mod tests {
    use time::macros::datetime;
    use crate::PrimitiveDateTimeExt;
    use crate::OffsetDateTimeExt;
    use crate::timezone;

    #[test]
    fn names() {
        //This test verifies that windows timezone names work fine.
        let shanghai = crate::timezones::get_by_name("Asia/Shanghai");
        let china = crate::timezones::get_by_name("China Standard Time");
        assert!(shanghai.is_some());
        assert!(china.is_some());
        assert_eq!(shanghai, china);
    }

    #[test]
    fn find() {
        let zones_iana = crate::timezones::find_by_name("Asia");
        let zones_win = crate::timezones::find_by_name("China Standard Time");
        assert!(zones_iana.len() > 1);
        assert!(zones_win.len() > 1);
    }

    #[test]
    fn london_to_berlin() {
        let dt = datetime!(2016-10-8 17:0:0).assume_timezone(timezone::europe::LONDON);
        let converted = dt.to_timezone(timezone::europe::BERLIN);
        let expected = datetime!(2016-10-8 18:0:0).assume_timezone(timezone::europe::BERLIN);
        assert_eq!(converted, expected);
    }
}

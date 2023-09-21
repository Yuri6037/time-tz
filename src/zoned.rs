// Copyright (c) 2023, Yuri6037
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

use time::{UtcOffset, PrimitiveDateTime, OffsetDateTime, Date, Time};

use crate::{TimeZone, OffsetResult, Offset, PrimitiveDateTimeExt};

#[derive(Clone, Copy)]
pub struct ZonedDateTime<'a, T: TimeZone> {
    date_time: PrimitiveDateTime,
    timezone: &'a T
}

impl<'a, T: TimeZone> ZonedDateTime<'a, T> {
    pub(crate) fn new(date_time: PrimitiveDateTime, timezone: &'a T) -> ZonedDateTime<'_, T> {
        ZonedDateTime {
            date_time,
            timezone
        }
    }

    /// Returns the date component of this ZonedDateTime.
    pub fn date(self) -> OffsetResult<Date> {
        self.to_offset_date_time().map(|v| v.date())
    }

    /// Returns the time component of this ZonedDateTime.
    pub fn time(self) -> OffsetResult<Time> {
        self.to_offset_date_time().map(|v| v.time())
    }

    /// Replaces the date component of this ZonedDateTime.
    pub fn replace_date(self, date: Date) -> ZonedDateTime<'a, T> {
        ZonedDateTime::new(self.date_time.replace_date(date), self.timezone)
    }

    /// Replaces the time component of this ZonedDateTime.
    pub fn replace_time(self, time: Time) -> ZonedDateTime<'a, T> {
        ZonedDateTime::new(self.date_time.replace_time(time), self.timezone)
    }

    /// Computes and returns the UTC offset of this ZonedDateTime.
    pub fn get_offset(&self) -> OffsetResult<UtcOffset> {
        self.timezone.get_offset_local(&self.date_time.assume_utc()).map(|v| v.to_utc())
    }

    /// Computes and returns the [OffsetDateTime](time::OffsetDateTime) from this ZonedDateTime.
    pub fn to_offset_date_time(self) -> OffsetResult<OffsetDateTime> {
        self.date_time.assume_timezone(self.timezone)
    }

    /// Returns the timezone component of this ZonedDateTime.
    pub fn timezone(self) -> &'a T {
        self.timezone
    }

    /// Replaces the timezone component of this ZonedDateTime.
    pub fn replace_timezone<'b, T1: TimeZone>(self, timezone: &'b T1) -> ZonedDateTime<'b, T1> {
        ZonedDateTime::new(self.date_time, timezone)
    }
}

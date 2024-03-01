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

use std::ops::{Add, Sub};

use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use crate::{OffsetResult, OffsetResultExt, PrimitiveDateTimeExt, TimeZone, ToTimezone};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct ZonedDateTime<'a, T: TimeZone> {
    date_time: OffsetDateTime,
    timezone: &'a T,
}

impl<'a, T: TimeZone> ZonedDateTime<'a, T> {
    pub fn from_local(
        date_time: PrimitiveDateTime,
        timezone: &'a T,
    ) -> OffsetResult<ZonedDateTime<'a, T>> {
        date_time
            .assume_timezone(timezone)
            .map_all(|v| ZonedDateTime {
                date_time: *v,
                timezone,
            })
    }

    pub fn from_utc(date_time: OffsetDateTime, timezone: &'a T) -> ZonedDateTime<'a, T> {
        let converted = date_time.to_timezone(timezone);
        ZonedDateTime {
            date_time: converted,
            timezone,
        }
    }

    fn from_local_offset(
        date_time: OffsetDateTime,
        timezone: &'a T,
    ) -> OffsetResult<ZonedDateTime<'a, T>> {
        let dt = PrimitiveDateTime::new(date_time.date(), date_time.time());
        dt.assume_timezone(timezone).map_all(|v| ZonedDateTime {
            date_time: *v,
            timezone,
        })
    }

    /// Returns the date component of this ZonedDateTime.
    pub fn date(self) -> Date {
        self.date_time.date()
    }

    /// Returns the time component of this ZonedDateTime.
    pub fn time(self) -> Time {
        self.date_time.time()
    }

    /// Replaces the date component of this ZonedDateTime.
    pub fn replace_date(self, date: Date) -> ZonedDateTime<'a, T> {
        ZonedDateTime {
            date_time: self.date_time.replace_date(date),
            timezone: self.timezone,
        }
    }

    /// Replaces the time component of this ZonedDateTime.
    pub fn replace_time(self, time: Time) -> OffsetResult<ZonedDateTime<'a, T>> {
        ZonedDateTime::from_local_offset(self.date_time.replace_time(time), self.timezone)
    }

    /// Computes and returns the UTC offset of this ZonedDateTime.
    pub fn offset(&self) -> UtcOffset {
        self.date_time.offset()
    }

    /// Computes and returns the [OffsetDateTime](time::OffsetDateTime) from this ZonedDateTime.
    pub fn offset_date_time(self) -> OffsetDateTime {
        self.date_time
    }

    /// Returns the timezone component of this ZonedDateTime.
    pub fn timezone(self) -> &'a T {
        self.timezone
    }

    /// Replaces the timezone component of this ZonedDateTime.
    pub fn replace_timezone<'b, T1: TimeZone>(self, timezone: &'b T1) -> ZonedDateTime<'b, T1> {
        ZonedDateTime::from_utc(self.date_time, timezone)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Duration {
    Date(time::Duration),
    Time(time::Duration),
}

impl From<time::Duration> for Duration {
    fn from(value: time::Duration) -> Self {
        Duration::Time(value)
    }
}

impl Duration {
    /// Create a new `Duration` with the given number of weeks.
    pub const fn weeks(weeks: i64) -> Self {
        Self::Date(time::Duration::weeks(weeks))
    }

    /// Create a new `Duration` with the given number of days.
    pub const fn days(days: i64) -> Self {
        Self::Date(time::Duration::days(days))
    }

    /// Create a new `Duration` with the given number of hours.
    pub const fn hours(hours: i64) -> Self {
        Self::Time(time::Duration::hours(hours))
    }

    /// Create a new `Duration` with the given number of minutes.
    pub const fn minutes(minutes: i64) -> Self {
        Self::Time(time::Duration::minutes(minutes))
    }

    /// Create a new `Duration` with the given number of seconds.
    pub const fn seconds(seconds: i64) -> Self {
        Self::Time(time::Duration::seconds(seconds))
    }
}

impl<'a, T: TimeZone> Add<Duration> for ZonedDateTime<'a, T> {
    type Output = ZonedDateTime<'a, T>;

    fn add(self, rhs: Duration) -> Self::Output {
        match rhs {
            Duration::Date(v) => {
                ZonedDateTime::from_local_offset(self.date_time + v, self.timezone).unwrap_first()
            }
            Duration::Time(v) => {
                let offset = self.offset();
                ZonedDateTime::from_local_offset(
                    self.date_time + v + time::Duration::seconds(offset.whole_seconds() as _),
                    self.timezone,
                )
                .unwrap_first()
            }
        }
    }
}

impl<'a, T: TimeZone> Sub<Duration> for ZonedDateTime<'a, T> {
    type Output = ZonedDateTime<'a, T>;

    fn sub(self, rhs: Duration) -> Self::Output {
        match rhs {
            Duration::Date(v) => {
                ZonedDateTime::from_local_offset(self.date_time - v, self.timezone).unwrap_first()
            }
            Duration::Time(v) => {
                let offset = self.offset();
                ZonedDateTime::from_local_offset(
                    self.date_time - v - time::Duration::seconds(offset.whole_seconds() as _),
                    self.timezone,
                )
                .unwrap_first()
            }
        }
    }
}

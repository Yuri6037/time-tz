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

use time::{OffsetDateTime, PrimitiveDateTime};

use crate::{timezones, zoned, Offset, OffsetError, OffsetResult, TimeZone, ToTimezone};

mod sealing {
    use crate::OffsetResult;

    pub trait OffsetDateTimeExt {}
    pub trait PrimitiveDateTimeExt {}
    pub trait OffsetResultExt {}

    impl OffsetDateTimeExt for time::OffsetDateTime {}
    impl PrimitiveDateTimeExt for time::PrimitiveDateTime {}
    impl<T> OffsetResultExt for OffsetResult<T> {}
}

// This trait is sealed and is only implemented in this library.
pub trait OffsetDateTimeExt: sealing::OffsetDateTimeExt {
    /// Converts this [OffsetDateTime](OffsetDateTime) to UTC.
    fn to_utc(&self) -> OffsetDateTime;

    /// Creates a new [ZonedDateTime](crate::ZonedDateTime) from this [OffsetDateTime](OffsetDateTime).
    ///
    /// # Arguments
    ///
    /// * `tz`: the target timezone.
    fn with_timezone<'a, T: TimeZone>(&self, tz: &'a T) -> zoned::ZonedDateTime<'a, T>;
}

/// This trait is sealed and is only implemented in this library.
pub trait PrimitiveDateTimeExt: sealing::PrimitiveDateTimeExt {
    /// Creates a new [OffsetDateTime](OffsetDateTime) from a [PrimitiveDateTime](PrimitiveDateTime) by assigning the main offset of the
    /// target timezone.
    ///
    /// *This assumes the [PrimitiveDateTime](PrimitiveDateTime) is already in the target timezone.*
    ///
    /// # Arguments
    ///
    /// * `tz`: the target timezone.
    ///
    /// returns: `OffsetResult<OffsetDateTime>`
    fn assume_timezone<T: TimeZone>(&self, tz: &T) -> OffsetResult<OffsetDateTime>;

    /// Creates a new [OffsetDateTime](OffsetDateTime) with the proper offset in the given timezone.
    ///
    /// *This assumes the [PrimitiveDateTime](PrimitiveDateTime) is in UTC offset.*
    ///
    /// # Arguments
    ///
    /// * `tz`: the target timezone.
    ///
    /// returns: OffsetDateTime
    fn assume_timezone_utc<T: TimeZone>(&self, tz: &T) -> OffsetDateTime;

    /// Creates a new [ZonedDateTime](crate::ZonedDateTime) from this [PrimitiveDateTime](PrimitiveDateTime).
    ///
    /// *This assumes the [PrimitiveDateTime](PrimitiveDateTime) is already in the target timezone.*
    ///
    /// # Arguments
    ///
    /// * `tz`: the target timezone.
    fn with_timezone<T: TimeZone>(self, tz: &T) -> OffsetResult<zoned::ZonedDateTime<T>>;
}

pub trait OffsetResultExt<T>: sealing::OffsetResultExt {
    /// Maps this [OffsetResult] to a different result type.
    fn map_all<R, F: Fn(&T) -> R>(&self, f: F) -> OffsetResult<R>;

    /// Unwraps this [OffsetResult] resolving ambiguity by taking the first result.
    fn unwrap_first(self) -> T;

    /// Unwraps this [OffsetResult] resolving ambiguity by taking the second result.
    fn unwrap_second(self) -> T;

    /// Turns this [OffsetResult] into an Option resolving ambiguity by taking the first result.
    fn take_first(self) -> Option<T>;

    /// Turns this [OffsetResult] into an Option resolving ambiguity by taking the second result.
    fn take_second(self) -> Option<T>;

    /// Returns true if this [OffsetResult] is neither ambiguous nor undefined.
    fn is_some(&self) -> bool;

    /// Returns true if this [OffsetResult] is None.
    fn is_none(&self) -> bool;

    /// Returns true if this [OffsetResult] is ambiguous.
    fn is_ambiguous(&self) -> bool;
}

impl PrimitiveDateTimeExt for PrimitiveDateTime {
    fn assume_timezone<T: TimeZone>(&self, tz: &T) -> OffsetResult<OffsetDateTime> {
        match tz.get_offset_local(&self.assume_utc()) {
            Ok(a) => Ok(self.assume_offset(a.to_utc())),
            Err(e) => match e {
                OffsetError::Ambiguous(a, b) => Err(OffsetError::Ambiguous(
                    self.assume_offset(a.to_utc()),
                    self.assume_offset(b.to_utc()),
                )),
                OffsetError::None => Err(OffsetError::None),
            },
        }
    }

    fn assume_timezone_utc<T: TimeZone>(&self, tz: &T) -> OffsetDateTime {
        let offset = tz.get_offset_utc(&self.assume_utc());
        self.assume_offset(offset.to_utc())
    }

    fn with_timezone<T: TimeZone>(self, tz: &T) -> OffsetResult<zoned::ZonedDateTime<T>> {
        zoned::ZonedDateTime::from_local(self, tz)
    }
}

impl OffsetDateTimeExt for OffsetDateTime {
    fn to_utc(&self) -> OffsetDateTime {
        if self.offset().is_utc() {
            *self
        } else {
            self.to_timezone(timezones::db::UTC)
        }
    }

    fn with_timezone<'a, T: TimeZone>(&self, tz: &'a T) -> zoned::ZonedDateTime<'a, T> {
        zoned::ZonedDateTime::from_utc(self.to_utc(), tz)
    }
}

impl<T: TimeZone> ToTimezone<&T> for OffsetDateTime {
    type Out = OffsetDateTime;
    type CheckedOut = Option<OffsetDateTime>;

    fn to_timezone(&self, tz: &T) -> OffsetDateTime {
        let offset = tz.get_offset_utc(self);
        self.to_offset(offset.to_utc())
    }

    fn checked_to_timezone(&self, tz: &T) -> Self::CheckedOut {
        let offset = tz.get_offset_utc(self);
        self.checked_to_offset(offset.to_utc())
    }
}

impl<T> OffsetResultExt<T> for OffsetResult<T> {
    fn map_all<R, F: Fn(&T) -> R>(&self, f: F) -> OffsetResult<R> {
        match self {
            Ok(a) => Ok(f(a)),
            Err(e) => Err(e.map(f)),
        }
    }

    fn unwrap_first(self) -> T {
        self.unwrap_or_else(|e| e.unwrap_first())
    }

    fn unwrap_second(self) -> T {
        self.unwrap_or_else(|e| e.unwrap_second())
    }

    fn take_first(self) -> Option<T> {
        match self {
            Ok(a) => Some(a),
            Err(e) => e.take_first(),
        }
    }

    fn take_second(self) -> Option<T> {
        match self {
            Ok(a) => Some(a),
            Err(e) => e.take_second(),
        }
    }

    fn is_some(&self) -> bool {
        self.is_ok()
    }

    fn is_none(&self) -> bool {
        match self {
            Ok(_) => false,
            Err(e) => e.is_none(),
        }
    }

    fn is_ambiguous(&self) -> bool {
        match self {
            Ok(_) => false,
            Err(e) => e.is_ambiguous(),
        }
    }
}

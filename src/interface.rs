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

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use time::{OffsetDateTime, UtcOffset};

/// This trait allows conversions from one timezone to another.
pub trait ToTimezone<T> {
    /// The output type.
    type Out;

    /// The output type for checked_to_timezone.
    type CheckedOut;

    /// Converts self to a different timezone.
    ///
    /// # Panics
    ///
    /// This function panics if the date_time + computed_offset would be out of range according to
    /// [OffsetDateTime](OffsetDateTime).
    fn to_timezone(&self, tz: T) -> Self::Out;

    /// Converts self to a different timezone.
    fn checked_to_timezone(&self, tz: T) -> Self::CheckedOut;
}

/// This trait represents a particular timezone offset.
pub trait Offset {
    /// Converts this timezone offset to a [UtcOffset](UtcOffset).
    fn to_utc(&self) -> UtcOffset;

    /// Returns the name of this offset.
    fn name(&self) -> &str;

    /// Returns true if this offset is DST in the corresponding timezone, false otherwise.
    fn is_dst(&self) -> bool;
}

/// This trait represents a timezone provider.
pub trait TimeZone {
    /// The type of offset.
    type Offset: Offset;

    /// Search for the given date time offset (assuming it is UTC) in this timezone.
    fn get_offset_utc(&self, date_time: &OffsetDateTime) -> Self::Offset;

    /// Search for the given date time offset (assuming it is already local) in this timezone.
    fn get_offset_local(&self, date_time: &OffsetDateTime) -> OffsetResult<Self::Offset>;

    /// Gets the main/default offset in this timezone.
    fn get_offset_primary(&self) -> Self::Offset;

    /// Returns the name of this timezone.
    fn name(&self) -> &str;
}

/// This represents the possible types of errors when trying to find a local offset.
#[derive(Clone, Copy, Debug)]
pub enum OffsetError<T> {
    /// The date time is ambiguous (2 offsets matches for the given date time in the given timezone).
    Ambiguous(T, T),

    /// No offset was found for the given date time in the given timezone.
    Undefined,
}

impl<T: Display> Display for OffsetError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OffsetError::Ambiguous(a, b) => write!(f, "multiple offsets matches for the given date time in the given timezone ({} and {})", a, b),
            OffsetError::Undefined => write!(f, "no offset found for the given date time in the given timezone")
        }
    }
}

impl<T: Display + Debug> Error for OffsetError<T> {}

impl<T> OffsetError<T> {
    /// Returns true if this [OffsetError] is undefined.
    pub fn is_undefined(&self) -> bool {
        match self {
            OffsetError::Ambiguous(_, _) => false,
            OffsetError::Undefined => true,
        }
    }

    /// Returns true if this [OffsetError] is ambiguous.
    pub fn is_ambiguous(&self) -> bool {
        match self {
            OffsetError::Ambiguous(_, _) => true,
            OffsetError::Undefined => false,
        }
    }

    /// Unwraps this [OffsetError] resolving ambiguity by taking the first result.
    pub fn unwrap_first(self) -> T {
        match self {
            OffsetError::Ambiguous(a, _) => a,
            OffsetError::Undefined => panic!("Attempt to unwrap an invalid offset"),
        }
    }

    /// Unwraps this [OffsetError] resolving ambiguity by taking the second result.
    pub fn unwrap_second(self) -> T {
        match self {
            OffsetError::Ambiguous(_, b) => b,
            OffsetError::Undefined => panic!("Attempt to unwrap an invalid offset"),
        }
    }

    /// Turns this [OffsetError] into an Option resolving ambiguity by taking the first result.
    pub fn take_first(self) -> Option<T> {
        match self {
            OffsetError::Ambiguous(a, _) => Some(a),
            OffsetError::Undefined => None,
        }
    }

    /// Turns this [OffsetError] into an Option resolving ambiguity by taking the second result.
    pub fn take_second(self) -> Option<T> {
        match self {
            OffsetError::Ambiguous(_, b) => Some(b),
            OffsetError::Undefined => None,
        }
    }

    /// Creates a reference to this [OffsetError].
    pub fn as_ref(&self) -> OffsetError<&T> {
        match self {
            OffsetError::Ambiguous(a, b) => OffsetError::Ambiguous(a, b),
            OffsetError::Undefined => OffsetError::Undefined,
        }
    }

    /// Maps this [OffsetError] to a different result type.
    pub fn map<R, F: Fn(&T) -> R>(self, f: F) -> OffsetError<R> {
        match self {
            OffsetError::Ambiguous(a, b) => OffsetError::Ambiguous(f(&a), f(&b)),
            OffsetError::Undefined => OffsetError::Undefined,
        }
    }
}

pub type OffsetResult<T> = Result<T, OffsetError<T>>;

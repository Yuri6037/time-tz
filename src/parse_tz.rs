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

use crate::TimeZone;
use nom::branch::alt;
use nom::bytes::complete::{is_not, take_while_m_n};
use nom::character::complete::{char as cchar, digit1};
use nom::combinator::{map_res, opt};
use nom::error::ErrorKind;
use nom::sequence::{delimited, tuple};
use nom::{Err, IResult};
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset, Weekday};

const TZNAME_MAX: usize = 16;

#[derive(Eq, PartialEq, Debug)]
struct Time {
    hh: u8,
    mm: Option<u8>,
    ss: Option<u8>,
}

impl Time {
    fn to_seconds(&self) -> u32 {
        self.hh as u32 * 3600 + self.mm.unwrap_or(0) as u32 * 60 + self.ss.unwrap_or(0) as u32
    }

    fn to_time(&self) -> time::Time {
        // Here unwrap should always pass because component range is checked in is_valid_range.
        time::Time::from_hms(self.hh, self.mm.unwrap_or(0), self.ss.unwrap_or(0)).unwrap()
    }

    fn is_valid_range(&self) -> bool {
        self.hh <= 24 && self.mm.unwrap_or(0) <= 59 && self.ss.unwrap_or(0) <= 59
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Offset {
    positive: bool,
    time: Time,
}

impl Offset {
    fn to_seconds(&self) -> i32 {
        match self.positive {
            true => self.time.to_seconds() as i32,
            false => -(self.time.to_seconds() as i32),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Date {
    J(u16),
    N(u16),
    M { m: u8, n: u8, d: u8 },
}

impl Date {
    fn to_date(&self, year: i32) -> time::Date {
        match self {
            Date::J(n) => {
                // Hack: the basic idea is 2021 was not a leap year so february only
                // contains 28 days instead of 29 which matches the POSIX spec.
                let date = time::Date::from_ordinal_date(2021, *n).unwrap();
                // Not sure if that will work in all cases though...
                time::Date::from_calendar_date(year, date.month(), date.day()).unwrap()
            }
            // ComponentRange errors should be prevented by is_valid_range.
            Date::N(n) => time::Date::from_ordinal_date(year, *n + 1).unwrap(),
            Date::M { m, n, d } => {
                // One more hack: here w're trying to match Date::from_iso_week_date.
                let week = m * n;
                let day = match d {
                    0 => Weekday::Sunday,
                    1 => Weekday::Monday,
                    2 => Weekday::Tuesday,
                    3 => Weekday::Wednesday,
                    4 => Weekday::Thursday,
                    5 => Weekday::Friday,
                    6 => Weekday::Saturday,
                    // SAFETY: This is basically impossible because d is a u8 so by definition
                    // cannot be < 0 and d <= 6 (see is_valid_range).
                    _ => unsafe { std::hint::unreachable_unchecked() },
                };
                // Here comes the interesting part, I'm not entirely sure unwrap is always gonna
                // pass here, this needs testing.
                time::Date::from_iso_week_date(year, week, day).unwrap()
            }
        }
    }

    fn is_valid_range(&self) -> bool {
        match self {
            Date::J(v) => (1..=365).contains(v),
            Date::N(v) => v <= &365,
            Date::M { m, n, d } => d <= &6 && (1..=5).contains(n) && (1..=12).contains(m),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Rule {
    start: (Date, Option<Time>),
    end: (Date, Option<Time>),
}

#[derive(Eq, PartialEq, Debug)]
struct Std<'a> {
    name: &'a str,
    offset: Offset,
}

#[derive(Eq, PartialEq, Debug)]
struct Dst<'a> {
    name: &'a str,
    offset: Option<Offset>,
    rule: Option<Rule>,
}

#[derive(Eq, PartialEq, Debug)]
enum Tz<'a> {
    Short(&'a str),
    Expanded { std: Std<'a>, dst: Option<Dst<'a>> },
}

impl<'a> Tz<'a> {
    fn ensure_valid_range(&self) -> Result<(), RangeError> {
        if let Tz::Expanded { std, dst } = self {
            if !std.offset.time.is_valid_range() {
                return Err(RangeError::Time);
            }
            if let Some(dst) = dst {
                if let Some(offset) = &dst.offset {
                    if !offset.time.is_valid_range() {
                        return Err(RangeError::Time);
                    }
                }
                if let Some(rule) = &dst.rule {
                    if !rule.start.0.is_valid_range() || !rule.end.0.is_valid_range() {
                        return Err(RangeError::Date);
                    }
                    if let Some(time) = &rule.start.1 {
                        if !time.is_valid_range() {
                            return Err(RangeError::Time);
                        }
                    }
                    if let Some(time) = &rule.end.1 {
                        if !time.is_valid_range() {
                            return Err(RangeError::Time);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub enum RangeError {
    Time,
    Date,
}

pub enum Error<'a> {
    Nom(ErrorKind),
    UnknownName(&'a str),
    Range(RangeError),
}

fn quoted_name(input: &str) -> IResult<&str, &str> {
    delimited(cchar('<'), is_not("<>"), cchar('>'))(input)
}

fn unquoted_name(input: &str) -> IResult<&str, &str> {
    take_while_m_n(3, TZNAME_MAX, |c: char| c.is_alphabetic())(input)
}

fn name(input: &str) -> IResult<&str, &str> {
    alt((quoted_name, unquoted_name))(input)
}

fn time_component(input: &str) -> IResult<&str, u8> {
    map_res(digit1, |v: &str| v.parse::<u8>())(input)
}

fn time_component_opt(input: &str) -> IResult<&str, u8> {
    let (v, (_, v1)) = tuple((cchar(':'), time_component))(input)?;
    Ok((v, v1))
}

fn sign(input: &str) -> IResult<&str, char> {
    alt((cchar('+'), cchar('-')))(input)
}

fn time(input: &str) -> IResult<&str, Time> {
    let (input, (hh, mm, ss)) = tuple((
        time_component,
        opt(time_component_opt),
        opt(time_component_opt),
    ))(input)?;
    Ok((input, Time { hh, mm, ss }))
}

fn time_opt(input: &str) -> IResult<&str, Time> {
    let (input, (_, hh, mm, ss)) = tuple((
        cchar('/'),
        time_component,
        opt(time_component_opt),
        opt(time_component_opt),
    ))(input)?;
    Ok((input, Time { hh, mm, ss }))
}

fn offset(input: &str) -> IResult<&str, Offset> {
    let (input, (sign, time)) = tuple((opt(sign), time))(input)?;
    let positive = sign.map(|v| v == '+').unwrap_or(true);
    Ok((input, Offset { positive, time }))
}

fn date_j(input: &str) -> IResult<&str, Date> {
    let (input, (_, n)) = tuple((cchar('J'), map_res(digit1, |v: &str| v.parse::<u16>())))(input)?;
    Ok((input, Date::J(n)))
}

fn date_n(input: &str) -> IResult<&str, Date> {
    let (input, n) = map_res(digit1, |v: &str| v.parse::<u16>())(input)?;
    Ok((input, Date::N(n)))
}

fn date_m(input: &str) -> IResult<&str, Date> {
    let (input, (_, m, _, n, _, d)) = tuple((
        cchar('M'),
        map_res(digit1, |v: &str| v.parse::<u8>()),
        cchar('.'),
        map_res(digit1, |v: &str| v.parse::<u8>()),
        cchar('.'),
        map_res(digit1, |v: &str| v.parse::<u8>()),
    ))(input)?;
    Ok((input, Date::M { m, n, d }))
}

fn date(input: &str) -> IResult<&str, Date> {
    alt((date_j, date_m, date_n))(input)
}

fn rule(input: &str) -> IResult<&str, Rule> {
    let (input, (_, start, _, end)) = tuple((
        cchar(','),
        tuple((date, opt(time_opt))),
        cchar(','),
        tuple((date, opt(time_opt))),
    ))(input)?;
    Ok((input, Rule { start, end }))
}

fn std(input: &str) -> IResult<&str, Std> {
    let (input, (name, offset)) = tuple((name, offset))(input)?;
    Ok((input, Std { name, offset }))
}

fn dst(input: &str) -> IResult<&str, Dst> {
    let (input, (name, offset, rule)) = tuple((name, opt(offset), opt(rule)))(input)?;
    Ok((input, Dst { name, offset, rule }))
}

fn tz_short(input: &str) -> IResult<&str, Tz> {
    let (input, (_, name)) = tuple((cchar(':'), name))(input)?;
    Ok((input, Tz::Short(name)))
}

fn tz_expanded(input: &str) -> IResult<&str, Tz> {
    let (input, (std, dst)) = tuple((std, opt(dst)))(input)?;
    Ok((input, Tz::Expanded { std, dst }))
}

fn entry(input: &str) -> IResult<&str, Tz> {
    alt((tz_short, tz_expanded))(input)
}

pub enum ParsedTzOffset<'a> {
    Existing(crate::timezone_impl::TzOffset),
    Expanded(&'a str, time::UtcOffset, bool),
}

impl<'a> crate::Offset for ParsedTzOffset<'a> {
    fn to_utc(&self) -> UtcOffset {
        match self {
            ParsedTzOffset::Existing(v) => v.to_utc(),
            ParsedTzOffset::Expanded(_, offset, _) => *offset,
        }
    }

    fn name(&self) -> &str {
        match self {
            ParsedTzOffset::Existing(v) => v.name(),
            ParsedTzOffset::Expanded(name, _, _) => name,
        }
    }

    fn is_dst(&self) -> bool {
        match self {
            ParsedTzOffset::Existing(v) => v.is_dst(),
            ParsedTzOffset::Expanded(_, _, v) => *v,
        }
    }
}

enum ParsedTz1<'a> {
    Existing(&'static crate::Tz),
    Expanded((Std<'a>, Option<Dst<'a>>)),
}

impl<'a> TimeZone for ParsedTz1<'a> {
    type Offset = ParsedTzOffset<'a>;

    fn get_offset_utc(&self, date_time: &OffsetDateTime) -> Self::Offset {
        match self {
            ParsedTz1::Existing(v) => ParsedTzOffset::Existing(v.get_offset_utc(date_time)),
            ParsedTz1::Expanded((std, dst)) => {
                let std_offset = UtcOffset::from_whole_seconds(std.offset.to_seconds()).unwrap();
                match dst {
                    None => {
                        //Easy case
                        ParsedTzOffset::Expanded(std.name, std_offset, false)
                    }
                    Some(dst) => {
                        // If the offset is undefined then assume +1 as specified by POSIX.
                        let offset = dst.offset.as_ref().map(|v| v.to_seconds()).unwrap_or(3600);
                        let dst_offset = UtcOffset::from_whole_seconds(offset).unwrap();
                        match &dst.rule {
                            None => {
                                use crate::Offset;
                                let timezone = crate::timezones::db::america::NEW_YORK;
                                let tz_offset = timezone.get_offset_utc(date_time);
                                if tz_offset.is_dst() {
                                    ParsedTzOffset::Expanded(dst.name, dst_offset, true)
                                } else {
                                    ParsedTzOffset::Expanded(std.name, std_offset, false)
                                }
                            }
                            Some(rule) => {
                                let start_date = rule.start.0.to_date(date_time.year());
                                let end_date = rule.end.0.to_date(date_time.year());
                                // SAFETY: This is forcely safe as never ever depends on user input.
                                let default =
                                    unsafe { time::Time::from_hms(2, 0, 0).unwrap_unchecked() };
                                let start_time = rule
                                    .start
                                    .1
                                    .as_ref()
                                    .map(|v| v.to_time())
                                    .unwrap_or(default);
                                let end_time =
                                    rule.end.1.as_ref().map(|v| v.to_time()).unwrap_or(default);
                                let start = PrimitiveDateTime::new(start_date, start_time)
                                    .assume_offset(std_offset);
                                let end = PrimitiveDateTime::new(end_date, end_time)
                                    .assume_offset(std_offset);
                                if date_time >= &start && date_time <= &end {
                                    // We are in DST mode.
                                    ParsedTzOffset::Expanded(dst.name, dst_offset, true)
                                } else {
                                    // We are in STD mode.
                                    ParsedTzOffset::Expanded(std.name, std_offset, false)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        match self {
            ParsedTz1::Existing(v) => v.name(),
            ParsedTz1::Expanded((v, _)) => v.name,
        }
    }
}

pub struct ParsedTz<'a> {
    inner: ParsedTz1<'a>,
}

impl<'a> TimeZone for ParsedTz<'a> {
    type Offset = ParsedTzOffset<'a>;

    fn get_offset_utc(&self, date_time: &OffsetDateTime) -> Self::Offset {
        self.inner.get_offset_utc(date_time)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

pub fn parse(input: &str) -> Result<ParsedTz, Error> {
    let (_, inner) = entry(input).map_err(|v| match v {
        Err::Incomplete(_) => {
            panic!("According to nom docs this case is impossible with complete API.")
        }
        Err::Error(e) => Error::Nom(e.code),
        Err::Failure(e) => Error::Nom(e.code),
    })?;
    inner.ensure_valid_range().map_err(Error::Range)?;
    let inner = match inner {
        Tz::Short(name) => {
            let tz = crate::timezones::find_by_name(name)
                .first()
                .copied()
                .ok_or(Error::UnknownName(name))?;
            ParsedTz1::Existing(tz)
        }
        Tz::Expanded { std, dst } => ParsedTz1::Expanded((std, dst)),
    };
    Ok(ParsedTz { inner })
}

#[cfg(test)]
mod tests {
    use crate::parse_tz::{entry, Date, Dst, Offset, Rule, Std, Time, Tz};

    #[test]
    fn basic() {
        let str = "ABC+1:00DEF,M1.2.3/4,56";
        let (_, test) = entry(str).unwrap();
        assert_eq!(
            test,
            Tz::Expanded {
                std: Std {
                    name: "ABC",
                    offset: Offset {
                        positive: true,
                        time: Time {
                            hh: 1,
                            mm: Some(0),
                            ss: None
                        }
                    }
                },
                dst: Some(Dst {
                    name: "DEF",
                    offset: None,
                    rule: Some(Rule {
                        start: (
                            Date::M { m: 1, n: 2, d: 3 },
                            Some(Time {
                                hh: 4,
                                mm: None,
                                ss: None
                            })
                        ),
                        end: (Date::N(56), None)
                    })
                })
            }
        )
    }
}

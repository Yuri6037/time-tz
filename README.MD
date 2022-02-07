# time-tz
An implementation of the tz database for the time-rs Rust crate.

This implementation is based off of chrono-tz (https://github.com/chronotope/chrono-tz) but uses time-rs instead
of chrono. This is designed to replace use of chono dependency which is impacted by CVE-2020-26235
(localtime_r thread safety issue linked to std::env::set_var).

**This is currently an experimental crate.**

## Features

- Injects an `assume_timezone` member function to any `PrimitiveDateTime`.
- Injects a `to_timezone` member function to any `OffsetDateTime`.
- Provides a `get` function to get a timezone by name.

## Usage

```rust
use time::macros::datetime;
use time_tz::{get, PrimitiveDateTimeExt, OffsetDateTimeExt};

fn main()
{
    // ===========================================
    //  Create a new datetime in a given timezone
    // ===========================================
    
    // First we have to get the source timezone:
    let london = get("Europe/London").unwrap();
    // The function returns None if the timezone doesn't exist.

    // Now we can create a primitive date time and call the extension function:
    let dt = datetime!(2016-10-8 17:0:0).assume_timezone(&london);

    
    // ===========================
    //  Convert to a new timezone
    // ===========================

    // First we get the target timezone:
    let berlin = get("Europe/Berlin").unwrap();

    // Now we can convert (again by calling an extension function):
    let converted = dt.to_timezone(&berlin);

    // ... do something with converted
}
```

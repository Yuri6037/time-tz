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

use crate::timezones::get_by_name;
use crate::Tz;

pub enum Error
{
    /// An IO error has occurred.
    Io(std::io::Error),

    /// An OS level error has occurred (can only happen on Windows).
    Os,

    /// The timezone is undetermined (means the timezone is not defined).
    Undetermined,

    /// The timezone doesn't exist in the crate's database.
    Unknown
}

pub fn get_timezone() -> Result<Tz, Error> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use std::path::Path;
            let path = Path::new("/etc/localtime");
            let realpath = std::fs::read_link(path).map_err(|v| Error::Io(v))?;
            // The part of the path we're interested in cannot contain non unicode characters.
            if let Some(iana) = realpath.to_string_lossy().split("/zoneinfo/").last() {
                let tz = get_by_name(iana).ok_or(Error::Unknown)?;
                return Ok(tz);
            } else {
                return Err(Error::Undetermined);
            }
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_timezone() {
        let tz = super::get_timezone();
        assert!(tz.is_ok());
    }
}

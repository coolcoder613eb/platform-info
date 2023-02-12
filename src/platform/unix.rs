// This file is part of the uutils coreutils package.
//
// (c) Jian Zeng <anonymousknight96 AT gmail.com>
// (c) Alex Lyon <arcterus@mail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

// spell-checker:ignore (API) domainname nodename osname sysname
// spell-checker:ignore (libc) libc utsname
// spell-checker:ignore (jargon) hasher
// spell-checker:ignore (names) Jian Zeng * anonymousknight96
// spell-checker:ignore (rust) uninit
// spell-checker:ignore (uutils) coreutils uutils
// spell-checker:ignore (VSCode) endregion

// refs:
// [Byte-to/from-String Conversions](https://nicholasbishop.github.io/rust-conversions) @@ <https://archive.is/AnDCY>

#![warn(unused_results)]

use std::borrow::Cow;
use std::cmp::Ordering;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use crate::PlatformInfoAPI;

use unix_safe::*;

// PlatformInfo
/// Handles initial retrieval and holds information for the current platform (a Unix-like OS in this case).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlatformInfo {
    // ref: <https://docs.rs/libc/latest/i686-unknown-linux-gnu/libc/struct.utsname.html>
    pub utsname: UTSName, /* aka "Unix Time-sharing System Name"; ref: <https://stackoverflow.com/questions/41669397/whats-the-meaning-of-utsname-in-linux> */
    // * private-use fields
    sysname: OsString,
    nodename: OsString,
    release: OsString,
    version: OsString,
    machine: OsString,
    osname: OsString,
}

impl PlatformInfo {
    /// Creates a new instance of `PlatformInfo`.
    /// This function *should* never fail.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let utsname = UTSName(utsname()?);
        Ok(Self {
            utsname,
            sysname: oss_from_cstr(&utsname.0.sysname),
            nodename: oss_from_cstr(&utsname.0.nodename),
            release: oss_from_cstr(&utsname.0.release),
            version: oss_from_cstr(&utsname.0.version),
            machine: oss_from_cstr(&utsname.0.machine),
            osname: OsString::from(crate::HOST_OS_NAME),
        })
    }
}

impl PlatformInfoAPI for PlatformInfo {
    fn sysname(&self) -> Result<Cow<str>, &OsString> {
        match self.sysname.to_str() {
            Some(str) => Ok(Cow::from(str)),
            None => Err(&self.sysname),
        }
    }

    fn nodename(&self) -> Result<Cow<str>, &OsString> {
        match self.nodename.to_str() {
            Some(str) => Ok(Cow::from(str)),
            None => Err(&self.nodename),
        }
    }

    fn release(&self) -> Result<Cow<str>, &OsString> {
        match self.release.to_str() {
            Some(str) => Ok(Cow::from(str)),
            None => Err(&self.release),
        }
    }

    fn version(&self) -> Result<Cow<str>, &OsString> {
        match self.version.to_str() {
            Some(str) => Ok(Cow::from(str)),
            None => Err(&self.version),
        }
    }

    fn machine(&self) -> Result<Cow<str>, &OsString> {
        match self.machine.to_str() {
            Some(str) => Ok(Cow::from(str)),
            None => Err(&self.machine),
        }
    }

    fn osname(&self) -> Result<Cow<str>, &OsString> {
        match self.osname.to_str() {
            Some(str) => Ok(Cow::from(str)),
            None => Err(&self.osname),
        }
    }
}

//===

// UTSName
/// Contains information about the current computer system.
///
/// Wraps [`libc::utsname`].
// ref: <https://docs.rs/libc/latest/i686-unknown-linux-gnu/libc/struct.utsname.html>
/*
    pub struct utsname {
        pub sysname: [::c_char; 65],
        pub nodename: [::c_char; 65],
        pub release: [::c_char; 65],
        pub version: [::c_char; 65],
        pub machine: [::c_char; 65],
        pub domainname: [::c_char; 65]
    }
*/
// aka "Unix Time-sharing System Name"; ref: <https://stackoverflow.com/questions/41669397/whats-the-meaning-of-utsname-in-linux>
#[derive(Clone, Copy /* , Debug, PartialEq, Eq, PartialOrd, Ord, Hash */)]
pub struct UTSName(libc::utsname);

impl Debug for UTSName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut debug_struct = &mut f.debug_struct("UTSName");
        debug_struct = debug_struct
            .field("sysname", &oss_from_cstr(&self.0.sysname))
            .field("nodename", &oss_from_cstr(&self.0.nodename))
            .field("release", &oss_from_cstr(&self.0.release))
            .field("version", &oss_from_cstr(&self.0.version))
            .field("machine", &oss_from_cstr(&self.0.machine));
        #[cfg(not(target_vendor = "apple" /* or `target_os = "macos"` */))]
        {
            debug_struct = debug_struct.field("domainname", &oss_from_cstr(&self.0.domainname));
        }
        debug_struct.finish()
    }
}

impl PartialEq for UTSName {
    fn eq(&self, other: &Self) -> bool {
        let mut equal = true; // avoid 'unused-mut' and 'clippy::let-and-return' warnings on MacOS
        equal = equal
            && (
                self.0.sysname,
                self.0.nodename,
                self.0.release,
                self.0.version,
                self.0.machine,
            ) == (
                other.0.sysname,
                other.0.nodename,
                other.0.release,
                other.0.version,
                other.0.machine,
            );
        #[cfg(not(target_vendor = "apple" /* or `target_os = "macos"` */))]
        {
            equal = equal && (self.0.domainname == other.0.domainname)
        }
        equal
    }
}

impl Eq for UTSName {}

impl PartialOrd for UTSName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UTSName {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut comparison = Ordering::Equal; // avoid 'unused-mut' and 'clippy::let-and-return' warnings on MacOS
        if comparison == Ordering::Equal {
            comparison = (
                self.0.sysname,
                self.0.nodename,
                self.0.release,
                self.0.version,
                self.0.machine,
            )
                .cmp(&(
                    other.0.sysname,
                    other.0.nodename,
                    other.0.release,
                    other.0.version,
                    other.0.machine,
                ));
        };
        #[cfg(not(target_vendor = "apple" /* or `target_os = "macos"` */))]
        {
            if comparison == Ordering::Equal {
                comparison = (self.0.domainname).cmp(&(other.0.domainname))
            }
        }
        comparison
    }
}

impl Hash for UTSName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.sysname.hash(state);
        self.0.nodename.hash(state);
        self.0.release.hash(state);
        self.0.version.hash(state);
        self.0.machine.hash(state);
        #[cfg(not(target_vendor = "apple" /* or `target_os = "macos"` */))]
        {
            self.0.domainname.hash(state);
        }
    }
}

//===

//#region unsafe code
mod unix_safe {
    use std::convert::TryFrom;
    use std::error::Error;
    use std::ffi::{CStr, OsStr, OsString};
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;

    // oss_from_str()
    /// *Returns* an OsString created from a libc::c_char slice.
    pub fn oss_from_cstr(slice: &[libc::c_char]) -> OsString {
        assert!(slice.len() < usize::try_from(isize::MAX).unwrap());
        assert!(slice.iter().position(|&c| c == 0 /* NUL */).unwrap() < slice.len());
        OsString::from(OsStr::from_bytes(
            unsafe { CStr::from_ptr(slice.as_ptr()) }.to_bytes(),
        ))
    }

    // utsname()
    /// *Returns* a `libc::utsname` structure containing `uname`-like OS system information.
    pub fn utsname() -> Result<libc::utsname, Box<dyn Error>> {
        // ref: <https://docs.rs/libc/latest/i686-unknown-linux-gnu/libc/fn.uname.html>
        // ref: <https://docs.rs/libc/latest/i686-unknown-linux-gnu/libc/struct.utsname.html>
        let mut uts = MaybeUninit::<libc::utsname>::uninit();
        let result = unsafe { libc::uname(uts.as_mut_ptr()) };
        if result != -1 {
            // SAFETY: `libc::uname()` succeeded => `uts` was initialized
            Ok(unsafe { uts.assume_init() })
        } else {
            Err(Box::new(io::Error::last_os_error()))
        }
    }
}
//#endregion (unsafe code)

//=== Tests

#[test]
fn test_osname() {
    let info = PlatformInfo::new().unwrap();
    let osname = match info.osname() {
        Ok(str) => {
            println!("osname = [{}]'{:?}'", str.len(), str);
            str
        }
        Err(os_s) => {
            let s = os_s.to_string_lossy();
            println!("osname = [{}]'{:?}' => '{}'", os_s.len(), os_s, s);
            Cow::from(String::from(s))
        }
    };
    assert!(osname.starts_with(crate::HOST_OS_NAME));
}

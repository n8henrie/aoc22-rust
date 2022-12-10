use std::{error, result};

pub type Error = Box<dyn error::Error + Send + Sync>;
pub type Result<T> = result::Result<T, Error>;

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => {{
        use $crate::errors::Error;
        Error::from(format!($($tt)*))
    }}
}

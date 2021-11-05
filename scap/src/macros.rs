macro_rules! mask {
    ( $expr:expr, $mask:expr ) => {
        $expr & $mask != 0
    }
}

macro_rules! not_null {
    ( $expr:expr, $on_err:expr $(,)? ) => {{
        let ret = unsafe { $expr };

        if ret.is_null() {
            Err($on_err(ret))
        } else {
            Ok(ret)
        }
    }}
}

macro_rules! not_negative {
    ( $expr:expr, $on_err:expr $(,)? ) => {{
        let ret = unsafe { $expr };

        if ret < 0 {
            Err($on_err(ret))
        } else {
            Ok(ret)
        }
    }}
}


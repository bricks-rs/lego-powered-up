#[macro_export]
macro_rules! ok {
    ($thing:expr) => {
        $thing.context("Cannot convert 'None'")?
    };
}

#[macro_export]
macro_rules! next {
    ($iter:ident) => {
        *$iter.next().context("Insufficient length")?
    };
}

macro_rules! four_bytes {
    ($t:ty, $iter:ident) => {
        <$t>::from_le_bytes([
            next!($iter),
            next!($iter),
            next!($iter),
            next!($iter),
        ])
    };
}

macro_rules! two_bytes {
    ($t:ty, $iter:ident) => {
        <$t>::from_le_bytes([next!($iter), next!($iter)])
    };
}

macro_rules! next_i32 {
    ($iter:ident) => {
        four_bytes!(i32, $iter)
    };
}

macro_rules! next_u32 {
    ($iter:ident) => {
        four_bytes!(u32, $iter)
    };
}

macro_rules! next_f32 {
    ($iter:ident) => {
        four_bytes!(f32, $iter)
    };
}

macro_rules! next_u16 {
    ($iter:ident) => {
        two_bytes!(u16, $iter)
    };
}

macro_rules! next_i16 {
    ($iter:ident) => {
        two_bytes!(i16, $iter)
    };
}

macro_rules! next_i8 {
    ($iter:ident) => {
        i8::from_le_bytes([next!($iter)])
    };
}
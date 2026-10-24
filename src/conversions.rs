//! Primitive wrapper conversions retain their native scalar representation.
macro_rules! scalar_conversion {
    ($message:path, $value:ty) => {
        impl From<$value> for $message {
            fn from(data: $value) -> Self {
                Self {
                    data,
                    ..Default::default()
                }
            }
        }
        impl From<$message> for $value {
            fn from(message: $message) -> Self {
                message.data
            }
        }
    };
}
scalar_conversion!(crate::stringmsg::StringMsg, String);
scalar_conversion!(crate::boolean::Boolean, bool);
scalar_conversion!(crate::int32::Int32, i32);
scalar_conversion!(crate::uint32::UInt32, u32);
scalar_conversion!(crate::int64::Int64, i64);
scalar_conversion!(crate::uint64::UInt64, u64);
scalar_conversion!(crate::float::Float, f32);
scalar_conversion!(crate::double::Double, f64);

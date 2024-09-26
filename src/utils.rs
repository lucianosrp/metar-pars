use nom::IResult;

#[macro_export]
macro_rules! tag_enum_parser {
    ( $( $tag:literal => $variant:expr ),* ) => {
        alt((
            $( value($variant, tag($tag)) ),*
        ))
    };
}

pub trait Parseable {
    fn parse(s: &str) -> IResult<&str, Self>
    where
        Self: Sized;
}
#[macro_export]
macro_rules! tag_enum {
    ($name:ident, $( $tag:literal => $variant:ident ),*) => {


        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
        pub enum $name {
            $( $variant ),*
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $( $tag => Ok($name::$variant), )*
                    _ => Err(Error::msg(format!("Cannot parse {}: {}", stringify!($name), s))),
                }
            }
        }

        impl $name {
             fn parse(s: &str) -> IResult<&str, $name> {
                alt((
                    $( value($name::$variant, tag($tag)), )*
                ))(s.trim_start())
            }
        }
    };
}

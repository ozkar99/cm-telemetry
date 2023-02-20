/// player_data implements the "player_data()" function
/// for the given impl_type, return_type and data_field
macro_rules! player_data {
    ($impl_type:ident, $return_type:ident, $data_field:ident) => {
        impl $impl_type {
            pub fn player_data(&self) -> &$return_type {
                let player_index = self.header.player_car_index as usize;
                &self.$data_field[player_index]
            }
        }
    };
}

pub(crate) use player_data;

/// binread_enum implements a default BinRead trait for enums
/// arguments are the enum to implement and the size of it
/// note: enum has to implement "Default" and "TryFromPrimitive" traits.
macro_rules! binread_enum {
    ($type:ident, $repr:ident) => {
        impl binread::BinRead for $type {
            type Args = ();
            fn read_options<R: binread::io::Read + binread::io::Seek>(
                reader: &mut R,
                options: &binread::ReadOptions,
                args: Self::Args,
            ) -> binread::BinResult<Self> {
                let byte = $repr::read_options(reader, options, args)?;
                Ok($type::try_from(byte).unwrap_or($type::default()))
            }
        }
    };
}

pub(crate) use binread_enum;
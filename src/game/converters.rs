pub struct ConversionError {}

pub type ConversionResult<T> = Result<T, ConversionError>;

fn convert_input_to_usize(input: &str) -> ConversionResult<usize> {
    input.parse::<usize>().map_err(|_| ConversionError {})
}

pub fn from_tcp_to_x_y<T>(input: T) -> ConversionResult<(usize, usize)>
where
    T: std::convert::AsRef<str>,
{
    let input = input.as_ref();

    if input.len() != 2 {
        return Err(ConversionError {});
    }

    let x = convert_input_to_usize(&input[0..1])?;
    let y = convert_input_to_usize(&input[1..2])?;
    Ok((x - 1, y - 1))
}

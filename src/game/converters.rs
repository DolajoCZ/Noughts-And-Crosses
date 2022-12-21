use super::playboard;

pub struct ConversionError {}

pub type ConversionResult<T> = Result<T, ConversionError>;

fn convert_input_to_usize(input: &str) -> ConversionResult<usize> {
    input.parse::<usize>().map_err(|_| ConversionError {})
}

pub fn pm_tcp_msg_to_x_y<T>(input: T) -> ConversionResult<(usize, usize)>
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

pub fn pb_3_3_to_string(field: &playboard::bp_3_3::Playboard) -> String {
    format!(
        " |1|2|3| \r\n\
         -|-|-|-|-\r\n\
         1|{}|{}|{}|1\r\n\
         -|-|-|-|-\r\n\
         2|{}|{}|{}|2\r\n\
         -|-|-|-|-\r\n\
         3|{}|{}|{}|3\r\n\
         -|-|-|-|-\r\n \
          |1|2|3| \r\n",
        field.fields[0],
        field.fields[1],
        field.fields[2],
        field.fields[3],
        field.fields[4],
        field.fields[5],
        field.fields[6],
        field.fields[7],
        field.fields[8]
    )
}

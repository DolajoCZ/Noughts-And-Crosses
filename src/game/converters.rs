use super::playboard;

pub struct ConversionError;

pub type ConversionResult<T> = Result<T, ConversionError>;

fn convert_input_to_usize(input: &str) -> ConversionResult<usize> {
    input
        .trim()
        .parse::<usize>()
        .map_err(|_| ConversionError {})
}

fn get_position_from_string(input: &str) -> ConversionResult<usize> {
    match convert_input_to_usize(input)?.checked_sub(1) {
        Some(x) => Ok(x),
        None => Err(ConversionError),
    }
}

pub fn pm_tcp_msg_to_x_y<T>(input: T) -> ConversionResult<(usize, usize)>
where
    T: std::convert::AsRef<str>,
{
    let mut inputs = input.as_ref().split('-');

    let mut x = 0_usize;
    let mut y = 0_usize;

    match inputs.next() {
        Some(value) => x = get_position_from_string(value)?,
        None => return Err(ConversionError {}),
    }

    match inputs.next() {
        Some(value) => y = get_position_from_string(value)?,

        None => return Err(ConversionError {}),
    }

    match inputs.next() {
        Some(_) => Err(ConversionError {}),
        None => Ok((x, y)),
    }
}

pub fn pb_n_n_to_string(playboard: &playboard::pb_n_n::Playboard) -> String {
    let mut rows: Vec<String> = Vec::with_capacity(playboard.edge_size + 2);

    // Rows separator
    let rows_separator: String = "\r\n".to_owned()
        + &(0..playboard.edge_size + 2)
            .map(|_| "-".to_owned())
            .collect::<Vec<String>>()
            .join("|")
        + "\r\n";

    // Create "header" for column indexes
    let column_indexes = " |".to_owned()
        + &(0..playboard.edge_size)
            .map(|x| format!("{}", x + 1))
            .collect::<Vec<String>>()
            .join("|")
        + "| ";

    rows.push(column_indexes.clone());

    // Push playboard field
    playboard
        .fields
        .windows(playboard.edge_size)
        .step_by(playboard.edge_size)
        .enumerate()
        .for_each(|(index, row)| {
            let middle = row
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<String>>()
                .join("|");

            rows.push(format!("{}|{}|{}", index + 1, middle, index + 1))
        });

    rows.push(column_indexes);

    rows.join(&rows_separator)
}

#[cfg(test)]
mod test {

    mod test_pb_n_n_to_string {
        use super::super::*;

        #[test]
        fn valid_case() {
            let playboard = playboard::pb_n_n::Playboard::new(5).unwrap();
            println!();
            println!();

            println!("{}", pb_n_n_to_string(&playboard));
        }
    }
}

// ---- Parsing input ----
enum ConvertError {
    InvalidInput,
    InvalidRange,
}

fn convert_input_to_coordinate(input: &str) -> Result<usize, ConvertError> {
    let value = match input.parse::<usize>() {
        Ok(x) => x,
        Err(_) => return Err(ConvertError::InvalidInput),
    };

    match value < 1 || value > 3 {
        true => Err(ConvertError::InvalidRange),
        false => Ok(value),
    }
}

fn convert_input_to_coordinates(input: &str) -> Result<[usize; 2], ConvertError> {
    if input.len() != 2 {
        return Err(ConvertError::InvalidInput);
    }

    let x = convert_input_to_coordinate(&input[0..1])?;
    let y = convert_input_to_coordinate(&input[1..2])?;
    Ok([x, y])
}

// ---- Get iteration over "lines" ----
fn get_iter_row(field: &Field, row_index: usize) -> std::slice::Iter<'_, Option<Symbols>> {
    field.fields[3 * row_index..3 * (row_index + 1)].iter()
}

fn get_iter_col_(
    field: &Field,
    col_index: usize,
) -> std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, Option<Symbols>>>> {
    field.fields.iter().skip(col_index).step_by(3)
}

fn get_iter_diag_neg(field: &Field) -> std::iter::StepBy<std::slice::Iter<'_, Option<Symbols>>> {
    field.fields.iter().step_by(4)
}

fn get_iter_diag_pos(field: &Field) -> std::iter::StepBy<std::slice::Iter<'_, Option<Symbols>>> {
    field.fields[2..7].iter().step_by(2)
}

// ---- Check field for win ----
fn are_same_some_values<'a, T>(mut data: T) -> bool
where
    T: Iterator<Item = &'a Option<Symbols>>,
{
    match data.next() {
        None => false,
        Some(x) => match x {
            Some(y) => data.all(|x| x == &Some(*y)),
            None => false,
        },
    }
}

fn check_for_win(field: &Field, position: usize) -> bool {
    // Check row and column
    if are_same_some_values(get_iter_row(field, position / 3))
        || are_same_some_values(get_iter_col_(field, position % 3))
    {
        return true;
    }

    // Check negative diagonal
    if (position == 0 || position == 4 || position == 8)
        && are_same_some_values(get_iter_diag_neg(field))
    {
        return true;
    }

    // Check positive diagonal
    if (position == 2 || position == 4 || position == 6)
        && are_same_some_values(get_iter_diag_pos(field))
    {
        return true;
    }

    false
}

// ---- Check field for draw ----

fn is_line_capable<'a, T>(mut data: T) -> bool
where
    T: Iterator<Item = &'a Option<Symbols>>,
{
    let score = data.fold(0_u8, |sum, item| {
        sum + match item {
            Some(x) => match x {
                Symbols::Circle => 1,
                Symbols::Cross => 10,
            },
            None => 100,
        }

        // sum + value
    });

    score == 12 || score == 21 || score == 111
}

fn check_for_draw(field: &Field) -> bool {
    !is_line_capable(get_iter_row(field, 0))
        && !is_line_capable(get_iter_row(field, 1))
        && !is_line_capable(get_iter_row(field, 2))
        && !is_line_capable(get_iter_col_(field, 0))
        && !is_line_capable(get_iter_col_(field, 1))
        && !is_line_capable(get_iter_col_(field, 2))
        && !is_line_capable(get_iter_diag_pos(field))
        && !is_line_capable(get_iter_diag_neg(field))
}

// ---- Field struct ----
#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
enum Symbols {
    Cross,
    Circle,
}

struct Field {
    fields: [Option<Symbols>; 9],
}

enum InvalidMove {
    InvalidInput,
    InvalidRange,
    AlreadyUsed,
}

enum ValidMove {
    Continue,
    Draw,
    Win,
}

impl Field {
    fn new() -> Field {
        Field {
            fields: [None, None, None, None, None, None, None, None, None],
        }
    }

    fn new_move(mut self, input: &str, symbol: Symbols) -> Result<ValidMove, InvalidMove> {
        // Parse coordinates
        let [x, y] = match convert_input_to_coordinates(input) {
            Ok(k) => k,
            Err(e) => match e {
                ConvertError::InvalidInput => return Err(InvalidMove::InvalidInput),
                ConvertError::InvalidRange => return Err(InvalidMove::InvalidRange),
            },
        };

        // Already taken
        if self.fields[3 * x + y].is_some() {
            return Err(InvalidMove::AlreadyUsed);
        };

        // Save data to field
        let position = 3 * x + y;
        self.fields[position] = Some(symbol);

        if check_for_win(&self, position) {
            return Ok(ValidMove::Win);
        }

        if check_for_draw(&self) {
            return Ok(ValidMove::Draw);
        }

        // let row_iter = get_row_iter(&self, 2);

        // are_same_some_values(row_iter);

        Ok(ValidMove::Continue)
    }
}

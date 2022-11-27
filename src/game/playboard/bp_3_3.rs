// ---- Parsing input ----
enum ConvertError {
    InvalidInput,
    InvalidRange,
}

#[derive(Clone, Copy)]
struct SingleField {
    field: Option<super::super::PlayerName>,
}

impl SingleField {
    fn new() -> Self {
        SingleField { field: None }
    }

    fn used_by_user(&self, player: &super::super::PlayerName) -> bool {
        match &self.field {
            Some(x) => x == player,
            None => false,
        }
    }
}

impl std::fmt::Display for SingleField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self.field {
                Some(x) => match x {
                    super::super::PlayerName::Circle => "o",
                    super::super::PlayerName::Cross => "x",
                },
                None => " ",
            }
        )
    }
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
    Ok([x - 1, y - 1])
}

// ---- Get iteration over "lines" ----
fn get_iter_row(field: &Playboard, row_index: usize) -> std::slice::Iter<'_, SingleField> {
    field.fields[3 * row_index..3 * (row_index + 1)].iter()
}

fn get_iter_col_(
    field: &Playboard,
    col_index: usize,
) -> std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, SingleField>>> {
    field.fields.iter().skip(col_index).step_by(3)
}

fn get_iter_diag_neg(field: &Playboard) -> std::iter::StepBy<std::slice::Iter<'_, SingleField>> {
    field.fields.iter().step_by(4)
}

fn get_iter_diag_pos(field: &Playboard) -> std::iter::StepBy<std::slice::Iter<'_, SingleField>> {
    field.fields[2..7].iter().step_by(2)
}

// ---- Check field for win ----
fn are_same_some_values<'a, T>(mut data: T) -> bool
where
    T: Iterator<Item = &'a SingleField>,
{
    match data.next() {
        None => false,
        Some(x) => match x.field {
            Some(y) => data.all(|x| x.used_by_user(&y)),
            None => false,
        },
    }
}

fn check_for_win(field: &Playboard, position: usize) -> bool {
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
fn is_line_capable<'a, T>(data: T) -> bool
where
    T: Iterator<Item = &'a SingleField>,
{
    let score = data.fold(0_u16, |sum, item| {
        sum + match item.field {
            Some(x) => match x {
                super::super::PlayerName::Circle => 1,
                super::super::PlayerName::Cross => 10,
            },
            None => 100,
        }
    });

    !(score == 12 || score == 21 || score == 111)
}

fn check_for_draw(field: &Playboard) -> bool {
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

pub struct Playboard {
    fields: [SingleField; 9],
}

impl Playboard {
    pub fn new() -> Playboard {
        Playboard {
            fields: [SingleField::new(); 9],
        }
    }
}

impl super::Playboard for Playboard {
    fn new_move(
        &mut self,
        input: &str,
        player_name: super::super::PlayerName,
    ) -> Result<super::ValidMove, super::InvalidMove> {
        // Parse coordinates
        let [x, y] = match convert_input_to_coordinates(input) {
            Ok(k) => k,
            Err(e) => match e {
                ConvertError::InvalidInput => return Err(super::InvalidMove::InvalidInput),
                ConvertError::InvalidRange => return Err(super::InvalidMove::InvalidRange),
            },
        };

        // Already taken
        if self.fields[3 * x + y].field.is_some() {
            return Err(super::InvalidMove::AlreadyUsed);
        };

        // Save data to field
        let position = 3 * x + y;
        self.fields[position] = SingleField {
            field: Some(player_name),
        };

        if check_for_win(&self, position) {
            return Ok(super::ValidMove::Win);
        }

        if check_for_draw(&self) {
            return Ok(super::ValidMove::Draw);
        }

        Ok(super::ValidMove::Continue)
    }
}

impl std::fmt::Display for Playboard {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " |1|2|3| \r\n\
             -|-|-|-|-\r\n\
             1|{}|{}|{}|1\r\n\
             -|-|-|-|-\r\n\
             2|{}|{}|{}|2\r\n\
             -|-|-|-|-\r\n\
             3|{}|{}|{}|3\r\n\
             -|-|-|-|-\r\n \
              |1|2|3| \r\n",
            self.fields[0],
            self.fields[1],
            self.fields[2],
            self.fields[3],
            self.fields[4],
            self.fields[5],
            self.fields[6],
            self.fields[7],
            self.fields[8]
        )
    }
}

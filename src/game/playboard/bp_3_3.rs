// ---- Parsing input ----
enum ConvertError {
    InvalidInput,
    InvalidRange,
}

#[derive(Clone, Copy)]
pub struct SingleField {
    pub field: Option<super::super::PlayerId>,
}

impl SingleField {
    fn new() -> Self {
        SingleField { field: None }
    }

    fn used_by_user(&self, player: &super::super::PlayerId) -> bool {
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
                    super::super::PlayerId::Circle => "o",
                    super::super::PlayerId::Cross => "x",
                },
                None => " ",
            }
        )
    }
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
                super::super::PlayerId::Circle => 1,
                super::super::PlayerId::Cross => 10,
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
    pub fields: [SingleField; 9],
}

impl Playboard {
    pub fn new() -> Playboard {
        Playboard {
            fields: [SingleField::new(); 9],
        }
    }
}

fn check_range(value: usize) -> bool {
    return value > 1 || value < 4;
}

impl super::Playboard for Playboard {
    type Position = (usize, usize);

    fn new_move(
        &mut self,
        position: Self::Position,
        player_id: super::super::PlayerId,
    ) -> Result<super::ValidMove, super::InvalidMove> {
        // Parse coordinates
        if !check_range(position.0) || !check_range(position.1) {
            return Err(super::InvalidMove::InvalidRange);
        }

        // Already taken
        if self.fields[3 * position.0 + position.1].field.is_some() {
            return Err(super::InvalidMove::AlreadyUsed);
        };

        // Save data to field
        let position = 3 * position.0 + position.1;
        self.fields[position] = SingleField {
            field: Some(player_id),
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

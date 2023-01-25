// ---- Parsing input ----
#[derive(Clone, Copy, Debug)]
pub struct SingleField {
    pub field: Option<super::super::PlayerId>,
}

/// Let something else
///
///
///
///
///
///
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
    field.fields[field.edge_size * row_index..field.edge_size * (row_index + 1)].iter()
}

fn get_iter_col_(
    field: &Playboard,
    col_index: usize,
) -> std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, SingleField>>> {
    field.fields.iter().skip(col_index).step_by(field.edge_size)
}

fn get_iter_diag_neg(field: &Playboard) -> std::iter::StepBy<std::slice::Iter<'_, SingleField>> {
    field.fields.iter().step_by(field.edge_size + 1)
}

fn get_iter_diag_pos(field: &Playboard) -> std::iter::StepBy<std::slice::Iter<'_, SingleField>> {
    field.fields[field.edge_size - 1..field.fields.len() - 1]
        .iter()
        .step_by(field.edge_size - 1)
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

fn check_for_win(field: &Playboard, position: (usize, usize)) -> bool {
    // Check row and column
    if are_same_some_values(get_iter_row(field, position.0))
        || are_same_some_values(get_iter_col_(field, position.1))
    {
        return true;
    }

    // Diagonal check
    if field.edge_size % 2 == 1 {
        // Check negative diagonal
        if (position.0 == position.1) && are_same_some_values(get_iter_diag_neg(field)) {
            return true;
        }

        // Check positive diagonal
        if ((position.0 + position.1) == field.edge_size - 1)
            && are_same_some_values(get_iter_diag_pos(field))
        {
            return true;
        }
    }

    false
}

// ---- Check field for draw ----
fn is_line_capable<'a, T>(data: T) -> bool
where
    T: Iterator<Item = &'a SingleField>,
{
    let mut user_cross = false;
    let mut user_circle = false;

    for item in data {
        match item.field {
            Some(x) => match x {
                super::super::PlayerId::Circle => user_circle = true,
                super::super::PlayerId::Cross => user_cross = true,
            },
            None => (),
        }
        if user_circle && user_cross {
            return false;
        }
    }

    true
}

fn check_for_draw(field: &Playboard) -> bool {
    let rows_capable =
        (0..field.edge_size).any(|index| is_line_capable(get_iter_row(field, index)));

    let columns_capable =
        (0..field.edge_size).any(|index| is_line_capable(get_iter_col_(field, index)));

    // Diagonal check
    if field.edge_size % 2 == 1 {
        return !(rows_capable
            || columns_capable
            || is_line_capable(get_iter_diag_neg(field))
            || is_line_capable(get_iter_diag_pos(field)));
    }

    !(rows_capable || columns_capable)
}

// ---- Field struct ----

#[derive(Debug)]
pub struct PlayboardToBig {}

impl std::fmt::Display for PlayboardToBig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Playboard nxn is to big");
    }
}

impl std::error::Error for PlayboardToBig {}

pub struct Playboard {
    pub fields: Vec<SingleField>,
    pub edge_size: usize,
}

impl Playboard {
    pub fn new(edge_size: usize) -> Result<Playboard, Box<dyn std::error::Error>> {
        match edge_size.checked_mul(edge_size) {
            Some(x) => Ok(Playboard {
                fields: (0..x).map(|_| SingleField::new()).collect(),
                edge_size: edge_size,
            }),
            None => Err(Box::new(PlayboardToBig {})),
        }
    }
}

fn check_range(value: usize, limit: usize) -> bool {
    value <= limit
}

impl super::Playboard for Playboard {
    type Position = (usize, usize);

    fn new_move(
        &mut self,
        position: Self::Position,
        player_id: super::super::PlayerId,
    ) -> Result<super::ValidMove, super::InvalidMove> {
        // Parse coordinates
        if !check_range(position.0, self.edge_size) || !check_range(position.1, self.edge_size) {
            return Err(super::InvalidMove::InvalidRange);
        }

        let position_ = self.edge_size * position.0 + position.1;

        // Already taken
        if self.fields[position_].field.is_some() {
            return Err(super::InvalidMove::AlreadyUsed);
        };

        // Save data to field
        self.fields[position_] = SingleField {
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

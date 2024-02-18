mod data;
mod lexer;
mod macros;

use anyhow::Result;

pub fn solve_problems(props: String) -> Result<bool> {
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_problems() {
        assert_eq!(solve_problems("".to_owned()).ok(), Some(true));
    }
}

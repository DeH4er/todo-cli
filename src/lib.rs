use args::{Args, Commands};
use commands::{
    add_command, clear_command, print_command, remove_command, set_done_command, AddCommandError,
    ClearCommandError, PrintCommandError, RemoveCommandError, SetDoneCommandError,
};
use db::{get_connection_with_table, GetConnectionWithTableError};

pub mod args;
mod commands;
mod db;
mod terminal;
mod todo;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum RunCommandError {
    #[error(transparent)]
    AddCommand(#[from] AddCommandError),

    #[error(transparent)]
    SetDoneCommand(#[from] SetDoneCommandError),

    #[error(transparent)]
    RemoveCommand(#[from] RemoveCommandError),

    #[error(transparent)]
    ClearCommand(#[from] ClearCommandError),

    #[error(transparent)]
    PrintAllCommand(#[from] PrintCommandError),

    #[error(transparent)]
    GetConnectionWithTable(#[from] GetConnectionWithTableError),
}

pub fn run_command(args: Args) -> Result<(), RunCommandError> {
    let mut connection = get_connection_with_table()?;

    match args.command {
        Some(Commands::Add { titles }) => {
            add_command(&mut connection, titles)?;
            print_command(&connection)?;
        }
        Some(Commands::Done { ids }) => {
            set_done_command(&mut connection, ids, true)?;
            print_command(&connection)?;
        }
        Some(Commands::Undone { ids }) => {
            set_done_command(&mut connection, ids, false)?;
            print_command(&connection)?;
        }
        Some(Commands::Remove { ids }) => {
            remove_command(&connection, ids)?;
            print_command(&connection)?;
        }
        Some(Commands::Clear) => {
            clear_command(&connection)?;
            print_command(&connection)?;
        }
        Some(Commands::Print) => print_command(&connection)?,
        None => print_command(&connection)?,
    };

    Ok(())
}

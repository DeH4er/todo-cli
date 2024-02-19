use rusqlite::Connection;

use crate::{
    db::{
        add_todos, get_todos, remove_todos, update_todos, AddTodosError, CreateTableError,
        GetTodosError, RemoveTodoError, UpdateTodosError,
    },
    terminal::strikethrough,
    todo::Todo,
};

#[derive(thiserror::Error, Debug)]
pub enum AddCommandError {
    #[error(transparent)]
    AddTodos(#[from] AddTodosError),

    #[error(transparent)]
    CreateTable(#[from] CreateTableError),
}

pub fn add_command(
    connection: &mut Connection,
    titles: Vec<String>,
) -> Result<(), AddCommandError> {
    let todos = titles.into_iter().map(Todo::new).collect();
    add_todos(connection, todos)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum SetDoneCommandError {
    #[error(transparent)]
    GetTodos(#[from] GetTodosError),

    #[error(transparent)]
    UpdateTodos(#[from] UpdateTodosError),
}

pub fn set_done_command(
    connection: &mut Connection,
    ids: Vec<usize>,
    done: bool,
) -> Result<(), SetDoneCommandError> {
    let todos = get_todos(&connection)?
        .into_iter()
        .enumerate()
        .filter(|(i, _)| ids.contains(&i))
        .map(|(_, todo)| Todo { done, ..todo })
        .collect();

    update_todos(connection, todos)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum RemoveCommandError {
    #[error(transparent)]
    GetTodos(#[from] GetTodosError),

    #[error(transparent)]
    RemoveTodos(#[from] RemoveTodoError),
}

pub fn remove_command(
    connection: &Connection,
    indexes: Vec<usize>,
) -> Result<(), RemoveCommandError> {
    let ids = get_todos(&connection)?
        .into_iter()
        .enumerate()
        .filter(|(i, _)| indexes.contains(&i))
        .map(|(_, todo)| todo.id)
        .collect();

    remove_todos(&connection, ids)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ClearCommandError {
    #[error(transparent)]
    GetTodos(#[from] GetTodosError),

    #[error(transparent)]
    RemoveTodos(#[from] RemoveTodoError),
}

pub fn clear_command(connection: &Connection) -> Result<(), ClearCommandError> {
    let ids = get_todos(&connection)?
        .into_iter()
        .filter(|todo| todo.done)
        .map(|todo| todo.id)
        .collect();

    remove_todos(&connection, ids)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum PrintCommandError {
    #[error(transparent)]
    CreateTable(#[from] CreateTableError),

    #[error(transparent)]
    GetTodos(#[from] GetTodosError),
}

pub fn print_command(connection: &Connection) -> Result<(), PrintCommandError> {
    let todos = get_todos(&connection)?;

    for (i, todo) in todos.iter().enumerate() {
        if todo.done {
            println!("{}: {}", i, strikethrough(&todo.title));
        } else {
            println!("{}: {}", i, &todo.title);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_table;
    use rusqlite::Connection;

    #[test]
    fn test_add_command() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_table(&mut connection).unwrap();

        let titles = vec!["title1".to_string(), "title2".to_string()];
        add_command(&mut connection, titles).unwrap();

        let todos = get_todos(&connection).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "title1");
        assert_eq!(todos[1].title, "title2");
    }

    #[test]
    fn test_set_done_command() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_table(&mut connection).unwrap();

        let titles = vec!["title1".to_string(), "title2".to_string()];
        add_command(&mut connection, titles).unwrap();

        let todos = get_todos(&connection).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].done, false);
        assert_eq!(todos[1].done, false);

        set_done_command(&mut connection, vec![0], true).unwrap();

        let todos = get_todos(&connection).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].done, true);
        assert_eq!(todos[1].done, false);
    }

    #[test]
    fn test_remove_command() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_table(&mut connection).unwrap();

        let titles = vec!["title1".to_string(), "title2".to_string()];
        add_command(&mut connection, titles).unwrap();

        let todos = get_todos(&connection).unwrap();
        assert_eq!(todos.len(), 2);

        remove_command(&connection, vec![0]).unwrap();

        let todos = get_todos(&connection).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "title2");
    }
}

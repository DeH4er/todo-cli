use std::rc::Rc;

use crate::{
    config::{get_db_path, GetDbPathError},
    todo,
};
use rusqlite::{types::Value, Connection};

const CREATE_TABLE_QUERY: &str = "CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    done BOOLEAN NOT NULL
)";

#[derive(thiserror::Error, Debug)]
#[error("Fail to get a todo")]
pub struct GetTodosError(#[from] rusqlite::Error);

pub fn get_todos(connection: &Connection) -> Result<Vec<todo::Todo>, GetTodosError> {
    let mut statement = connection.prepare("SELECT id, title, done FROM todos")?;
    let todos = statement
        .query_map([], |row| {
            Ok(todo::Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                done: row.get(2)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(todos)
}

#[derive(thiserror::Error, Debug)]
pub enum AddTodosError {
    #[error("Fail to create transaction")]
    CreateTransaction(#[source] rusqlite::Error),

    #[error("Fail to prepare insert statement")]
    PrepareInsert(#[source] rusqlite::Error),

    #[error("Fail to insert todo")]
    InsertTodo(#[source] rusqlite::Error),

    #[error("Fail to commit transaction")]
    CommitTransaction(#[source] rusqlite::Error),
}

pub fn add_todos(connection: &mut Connection, todos: Vec<todo::Todo>) -> Result<(), AddTodosError> {
    let transaction = connection
        .transaction()
        .map_err(AddTodosError::CreateTransaction)?;

    {
        let mut statement = transaction
            .prepare("INSERT INTO todos (title, done) VALUES (?1, ?2)")
            .map_err(AddTodosError::PrepareInsert)?;

        for todo in todos {
            statement
                .execute(rusqlite::params![todo.title, todo.done])
                .map_err(AddTodosError::InsertTodo)?;
        }
    }

    transaction
        .commit()
        .map_err(AddTodosError::CommitTransaction)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateTodosError {
    #[error("Fail to create transaction")]
    CreateTransaction(#[source] rusqlite::Error),

    #[error("Fail to create statement")]
    Statement(#[source] rusqlite::Error),

    #[error("Fail to update todo")]
    UpdateTodo(#[source] rusqlite::Error),

    #[error("Fail to commit transaction")]
    CommitTransaction(#[source] rusqlite::Error),
}

pub fn update_todos(
    connection: &mut Connection,
    todos: Vec<todo::Todo>,
) -> Result<(), UpdateTodosError> {
    let transaction = connection
        .transaction()
        .map_err(UpdateTodosError::CreateTransaction)?;

    {
        let mut statement = transaction
            .prepare("UPDATE todos SET title = ?1, done = ?2 WHERE id = ?3")
            .map_err(UpdateTodosError::Statement)?;

        for todo in todos {
            statement
                .execute(rusqlite::params![todo.title, todo.done, todo.id])
                .map_err(UpdateTodosError::UpdateTodo)?;
        }
    }

    transaction
        .commit()
        .map_err(UpdateTodosError::CommitTransaction)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
#[error("Fail to remove todo")]
pub struct RemoveTodoError(#[from] rusqlite::Error);

pub fn remove_todos(connection: &Connection, ids: Vec<usize>) -> Result<(), RemoveTodoError> {
    let ids: Vec<Value> = ids.into_iter().map(|id| Value::from(id as u32)).collect();
    let rc = Rc::new(ids);

    connection.execute(
        "DELETE FROM todos WHERE id in rarray(?1)",
        rusqlite::params![rc],
    )?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum GetConnectionError {
    #[error("Fail to create and connect to a db")]
    Open(#[from] rusqlite::Error),

    #[error(transparent)]
    GetDbPath(#[from] GetDbPathError),
}

pub fn get_connection() -> Result<Connection, GetConnectionError> {
    let connection = Connection::open(get_db_path()?)?;

    Ok(connection)
}

#[derive(thiserror::Error, Debug)]
pub enum CreateTableError {
    #[error("Fail to load array module")]
    LoadArrayModule(#[source] rusqlite::Error),

    #[error("Fail to execute create table query")]
    ExecuteCreateTableQuery(#[source] rusqlite::Error),
}

pub fn create_table(connection: &Connection) -> Result<(), CreateTableError> {
    rusqlite::vtab::array::load_module(&connection).map_err(CreateTableError::LoadArrayModule)?;
    connection
        .execute(CREATE_TABLE_QUERY, [])
        .map_err(CreateTableError::ExecuteCreateTableQuery)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum GetConnectionWithTableError {
    #[error(transparent)]
    GetConnection(#[from] GetConnectionError),

    #[error(transparent)]
    CreateTable(#[from] CreateTableError),
}

pub fn get_connection_with_table() -> Result<Connection, GetConnectionWithTableError> {
    let connection = get_connection()?;
    create_table(&connection)?;
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use self::todo::Todo;

    use super::*;
    use rusqlite::params;

    #[test]
    fn test_create_table() {
        let connection = Connection::open_in_memory().unwrap();
        create_table(&connection).unwrap();

        let table_info = connection
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='todos'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<String>>();

        assert_eq!(table_info.len(), 1);
        assert_eq!(table_info[0], "todos");
    }

    #[test]
    fn test_get_todos() {
        let connection = Connection::open_in_memory().unwrap();
        create_table(&connection).unwrap();

        let todos = get_todos(&connection).unwrap();
        assert_eq!(todos.len(), 0);

        connection
            .execute(
                "INSERT INTO todos (title, done) VALUES (?1, ?2)",
                params!["todo1", false],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO todos (title, done) VALUES (?1, ?2)",
                params!["todo2", true],
            )
            .unwrap();

        let todos = get_todos(&connection).unwrap();

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "todo1");
        assert_eq!(todos[0].done, false);
        assert_eq!(todos[1].title, "todo2");
        assert_eq!(todos[1].done, true);
    }

    #[test]
    fn test_add_todos() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_table(&connection).unwrap();

        let expected_todos = vec![Todo::new("todo1".into()), Todo::new("todo2".into())];

        add_todos(&mut connection, expected_todos.clone()).unwrap();

        let received_todos = get_todos(&connection).unwrap();

        assert_eq!(received_todos.len(), expected_todos.len());

        for (received, expected) in received_todos.iter().zip(expected_todos.iter()) {
            assert_eq!(received.title, expected.title);
            assert_eq!(received.done, expected.done);
        }
    }

    #[test]
    fn test_update_todos() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_table(&connection).unwrap();

        connection
            .execute(
                "INSERT INTO todos (title, done) VALUES (?1, ?2)",
                params!["todo1", false],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO todos (title, done) VALUES (?1, ?2)",
                params!["todo2", true],
            )
            .unwrap();

        let mut todos = get_todos(&connection).unwrap();
        todos[0].title = "new todo1".into();
        todos[0].done = true;
        todos[1].title = "new todo2".into();
        todos[1].done = false;

        update_todos(&mut connection, todos).unwrap();

        let received_todos = get_todos(&connection).unwrap();

        assert_eq!(received_todos.len(), 2);
        assert_eq!(received_todos[0].title, "new todo1");
        assert_eq!(received_todos[0].done, true);
        assert_eq!(received_todos[1].title, "new todo2");
        assert_eq!(received_todos[1].done, false);
    }

    #[test]
    fn test_remove_todos() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_table(&connection).unwrap();

        connection
            .execute(
                "INSERT INTO todos (id, title, done) VALUES (?1, ?2, ?3)",
                params![0, "todo1", false],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO todos (id, title, done) VALUES (?1, ?2, ?3)",
                params![1, "todo2", true],
            )
            .unwrap();

        remove_todos(&mut connection, vec![0]).unwrap();

        let todos = get_todos(&connection).unwrap();

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "todo2");
        assert_eq!(todos[0].done, true);
    }
}

use chrono::{self, Utc};
use cli_table::{format::Justify, Cell, Style, Table};
use core::fmt;
use rusqlite::Connection;
use std::io;
use std::path::Path;

#[derive(Debug)]
enum TaskStatus {
    Todo = 0,
    Doing = 1,
    Cancelled = 2,
    Completed = 3,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct Task {
    id: Option<i32>,
    summary: String,
    assignee: String,
    status: TaskStatus,
    created: String,
}

#[repr(i8)]
#[derive(Debug)]
enum MenuItem {
    ShowTasks = 0,
    AddTask(Task) = 1,
}

fn main() {
    db_init();
    clear_screen();
    let selected_item: MenuItem = show_menu();

    match selected_item {
        MenuItem::ShowTasks => {
            clear_screen();
            db_show_tasks();
        }
        MenuItem::AddTask(t) => db_add_task(t),
    }
}

fn show_menu() -> MenuItem {
    println!("Choose one of items");
    println!("1) : Show all tasks");
    println!("2) : Add new task");

    let selected_item: MenuItem = loop {
        let mut selected_item = String::new();
        io::stdin()
            .read_line(&mut selected_item)
            .expect("Invalid Item");

        let selected_item: i32 = match selected_item.trim().parse() {
            Ok(num) => num,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        let selected_item: MenuItem = match selected_item {
            1 => MenuItem::ShowTasks,
            2 => {
                clear_screen();
                MenuItem::AddTask(add_task())
            }
            _ => {
                println!("Invalid order . try again");
                continue;
            }
        };

        break selected_item;
    };

    return selected_item;
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn add_task() -> Task {
    println!("Enter the task summary:");

    let mut new_task_summary = String::new();
    io::stdin()
        .read_line(&mut new_task_summary)
        .expect("Couldn't parse the given task summary");

    println!("Enter the task assignee:");

    let mut new_task_assignee = String::new();
    io::stdin()
        .read_line(&mut new_task_assignee)
        .expect("Couldn't parse the given task summary");

    let new_task_time = String::from(Utc::now().to_string());

    let new_task: Task = Task {
        id: None,
        summary: String::from(new_task_summary.trim()),
        assignee: String::from(new_task_assignee.trim()),
        created: new_task_time,
        status: TaskStatus::Todo,
    };

    return new_task;
}

//////////////////////
/// DATABASE CALLS ///
//////////////////////
fn db_get_connection() -> Connection {
    Connection::open("tasks.db").expect("Couldn't open the db connection")
}

fn db_init() {
    let is_exists = Path::new("tasks.db").exists();
    if is_exists != true {
        println!("Initializing database");
        db_initial_tables();
    }
}

fn db_initial_tables() {
    let db_conn = db_get_connection();
    //Handle and refactor this part
    let a = db_conn.execute(
        "create table if not exists tasks(
        id integer PRIMARY KEY,
        summary text NOT NULL,
        assignee text NOT NULL,
        status integer NOT NULL,
        Date text NOT NULL);",
        (),
    );

    match a {
        Ok(s) => println!("{}", s),
        Err(e) => println!("{}", e),
    }
}

fn db_add_task(new_task: Task) {
    let db_conn = db_get_connection();

    let status: i8 = new_task.status as i8;

    let insert_task_result = db_conn.execute(
        "insert into tasks (summary,assignee,status,date)
    VALUES (?1,?2,?3,?4)",
        (
            new_task.summary,
            new_task.assignee,
            status,
            new_task.created,
        ),
    );

    match insert_task_result {
        Ok(_) => println!("The task added successfully"),
        Err(_) => println!("error"),
    }
}

fn db_show_tasks() {
    let db_conn = db_get_connection();

    let stmt = db_conn.prepare("SELECT id,summary,assignee,status,date FROM tasks");

    if stmt.is_ok() {
        let mut statement = stmt.unwrap();

        let task_iter = statement.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                summary: row.get(1)?,
                assignee: row.get(2)?,
                status: match row.get(3)? {
                    0 => TaskStatus::Todo,
                    1 => TaskStatus::Doing,
                    2 => TaskStatus::Cancelled,
                    3 => TaskStatus::Completed,
                    _ => panic!("Couldn't find matched value for task status type"),
                },
                created: row.get(4)?,
            })
        });

        let mut table = vec![];
        for i in task_iter.unwrap() {
            let task = i.unwrap();

            table.push(vec![
                task.id.unwrap().cell().justify(Justify::Center),
                task.summary.cell().justify(Justify::Center),
                task.assignee.cell().justify(Justify::Center),
                task.status.to_string().cell().justify(Justify::Center),
            ]);
        }

        let table = table
            .table()
            .title(vec![
                "Id".cell()
                    .bold(true)
                    .foreground_color(Some(cli_table::Color::Blue)),
                "Summary"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(cli_table::Color::Blue)),
                "Assignee"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(cli_table::Color::Blue)),
                "Status"
                    .cell()
                    .bold(true)
                    .foreground_color(Some(cli_table::Color::Blue)),
            ])
            .bold(true)
            .display()
            .unwrap();

        println!("{}", table);
    }
}

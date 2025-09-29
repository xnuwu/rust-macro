use macros::{Getter, timed, Entity, concat_strings};
use std::time::{Duration};

#[timed]
fn my_sleep(ms: Duration) -> i32 {
    std::thread::sleep(ms);
    return 20;
}

macro_rules! create_struct_with_getter {
    ($name:ident { $($field:ident : $ty:ty),* }) => {
        #[derive(Getter, Debug, Entity)]
        struct $name {
            $($field: $ty),*
        }
    };
}

create_struct_with_getter!(Person {name: String, age: i32});

// Enhanced struct with various types and ORM attributes
#[derive(Getter, Debug, Entity)]
struct User {
    #[orm(primary_key)]
    id: i64,
    username: String,
    email: String,
    age: Option<i32>,
    score: f64,
    is_active: bool,
    balance: Option<f32>,
}

fn main() {
    let person = Person {
        name: String::from("ss"),
        age: 25,
    };
    
    println!("=== Person Example ===");
    println!("Name: {}, Age: {}", person.name(), person.age());
    println!("Table: {}", Person::table_name());
    println!("Create Table: {}", Person::create_table());
    println!("Insert SQL: {}", person.insert());
    println!("Primary Keys: {:?}", Person::primary_keys());
    
    println!("\n=== User Example (Enhanced) ===");
    let user = User {
        id: 1,
        username: String::from("john_doe"),
        email: String::from("john@example.com"),
        age: Some(30),
        score: 95.5,
        is_active: true,
        balance: None,
    };
    
    println!("User ID: {}, Username: {}", user.id(), user.username());
    println!("Email: {}, Active: {}", user.email(), user.is_active());
    println!("Age: {:?}, Score: {}, Balance: {:?}", user.age(), user.score(), user.balance());
    println!("Table: {}", User::table_name());
    println!("Create Table: {}", User::create_table());
    println!("Insert SQL: {}", user.insert());
    println!("Primary Keys: {:?}", User::primary_keys());
    
    my_sleep(std::time::Duration::from_millis(100));

    println!("=== Concat Strings Example ===");
    concat_strings!(("hello", " ", "world", "!"));
    println!("Concat Strings: {}", CONCATENATED);
}
use std::process::Command;
use colored::*;

fn main() {
    println!("מתחיל תהליך העלאה לענף upgraded-version...");

    // בדיקה אם הענף קיים
    let branch_exists = Command::new("git")
        .args(&["branch", "--list", "upgraded-version"])
        .output()
        .expect("שגיאה בבדיקת קיום הענף")
        .stdout
        .len() > 0;

    if !branch_exists {
        println!("יוצר ענף upgraded-version חדש...");
        if let Err(e) = Command::new("git")
            .args(&["checkout", "-b", "upgraded-version"])
            .status() {
            eprintln!("{}", format!("שגיאה ביצירת הענף: {}", e).red());
            std::process::exit(1);
        }
    } else {
        println!("עובר לענף upgraded-version...");
        if let Err(e) = Command::new("git")
            .args(&["checkout", "upgraded-version"])
            .status() {
            eprintln!("{}", format!("שגיאה במעבר לענף: {}", e).red());
            std::process::exit(1);
        }
    }

    // משיכת שינויים מהשרת
    println!("מושך שינויים מהשרת...");
    if let Err(e) = Command::new("git")
        .args(&["pull", "origin", "upgraded-version"])
        .status() {
        eprintln!("{}", format!("שגיאה במשיכת שינויים: {}", e).red());
        // ממשיכים גם אם יש שגיאה במשיכה, כי ייתכן שהענף לא קיים בשרת
    }

    // הוספת כל השינויים
    println!("מוסיף את כל השינויים...");
    if let Err(e) = Command::new("git")
        .args(&["add", "."])
        .status() {
        eprintln!("{}", format!("שגיאה בהוספת שינויים: {}", e).red());
        std::process::exit(1);
    }

    // יצירת קומיט
    println!("יוצר קומיט...");
    if let Err(e) = Command::new("git")
        .args(&["commit", "-m", "עדכון גרסה"])
        .status() {
        eprintln!("{}", format!("שגיאה ביצירת קומיט: {}", e).red());
        std::process::exit(1);
    }

    // דחיפת שינויים לשרת
    println!("דוחף שינויים לשרת...");
    let push_result = Command::new("git")
        .args(&["push", "-u", "origin", "upgraded-version"])
        .output()
        .expect("שגיאה בדחיפה לשרת");

    if !push_result.status.success() {
        let error_message = String::from_utf8_lossy(&push_result.stderr);
        eprintln!("{}", format!("שגיאה בדחיפה לשרת: {}", error_message).red());
        std::process::exit(1);
    }

    println!("{}", "העלאה הושלמה בהצלחה!".green());
} 
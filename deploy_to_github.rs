use std::process::Command;
use std::env;

fn main() {
    // בדיקה שאנחנו בתיקיית הפרויקט הנכונה
    if !std::path::Path::new("Cargo.toml").exists() {
        eprintln!("Error: Cargo.toml not found. Please run from project root.");
        std::process::exit(1);
    }

    // אתחול רפוזיטורי git אם לא קיים
    if !std::path::Path::new(".git").exists() {
        Command::new("git")
            .args(&["init"])
            .status()
            .expect("Failed to initialize git repository");
    }

    // הוספת כל הקבצים
    Command::new("git")
        .args(&["add", "."])
        .status()
        .expect("Failed to add files");

    // יצירת commit
    Command::new("git")
        .args(&["commit", "-m", "Update project files"])
        .status()
        .expect("Failed to commit changes");

    // הוספת remote אם לא קיים
    Command::new("git")
        .args(&["remote", "add", "origin", "https://github.com/username/RustoHebru.git"])
        .status()
        .expect("Failed to add remote");

    // דחיפה לgithub
    Command::new("git")
        .args(&["push", "-u", "origin", "main"])
        .status()
        .expect("Failed to push to GitHub");

    println!("Successfully deployed to GitHub!");
} 
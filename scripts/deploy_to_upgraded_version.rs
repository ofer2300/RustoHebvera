use std::process::{Command, Output};
use std::error::Error;
use colored::*;

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", "מתחיל תהליך העלאה לענף upgraded-version...".green());

    // בדיקה אם אנחנו בתיקיית הפרויקט
    if !std::path::Path::new("Cargo.toml").exists() {
        return Err("שגיאה: יש להריץ את הסקריפט מתיקיית הפרויקט הראשית".into());
    }

    // בדיקה אם קיים רפוזיטורי git
    if !std::path::Path::new(".git").exists() {
        println!("{}", "מאתחל רפוזיטורי git חדש...".yellow());
        run_command("git", &["init"])?;
    }

    // בדיקת חיבור לרפוזיטורי מרוחק
    let remote_exists = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !remote_exists {
        println!("{}", "מוסיף remote למאגר...".yellow());
        run_command(
            "git",
            &["remote", "add", "origin", "https://github.com/ofer2300/RustoHebvera.git"],
        )?;
    }

    // בדיקה אם הענף upgraded-version קיים
    let branch_exists = Command::new("git")
        .args(&["branch", "--list", "upgraded-version"])
        .output()?
        .stdout
        .len() > 0;

    if !branch_exists {
        println!("{}", "יוצר ענף upgraded-version חדש...".yellow());
        run_command("git", &["checkout", "-b", "upgraded-version"])?;
    } else {
        println!("{}", "עובר לענף upgraded-version...".yellow());
        run_command("git", &["checkout", "upgraded-version"])?;
    }

    // הוספת כל הקבצים החדשים והשינויים
    println!("{}", "מוסיף את כל השינויים...".yellow());
    run_command("git", &["add", "."])?;

    // בדיקה אם יש שינויים להעלות
    let status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;

    if status.stdout.is_empty() {
        println!("{}", "אין שינויים להעלות".blue());
        return Ok(());
    }

    // יצירת קומיט
    let commit_message = format!(
        "עדכון גרסה {} - {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        "שדרוג מערכת התרגום והוספת בקרת איכות"
    );
    
    println!("{}", "יוצר קומיט...".yellow());
    run_command("git", &["commit", "-m", &commit_message])?;

    // דחיפה לשרת
    println!("{}", "דוחף שינויים לשרת...".yellow());
    let push_result = Command::new("git")
        .args(&["push", "-u", "origin", "upgraded-version"])
        .output();

    match push_result {
        Ok(output) => {
            if output.status.success() {
                println!("{}", "העלאה הושלמה בהצלחה!".green());
                println!("הענף upgraded-version עודכן בהצלחה");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("שגיאה בדחיפה לשרת: {}", error).into());
            }
        }
        Err(e) => {
            return Err(format!("שגיאה בדחיפה לשרת: {}", e).into());
        }
    }

    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<Output, Box<dyn Error>> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("שגיאה בהרצת פקודה {}: {}", cmd, error).into());
    }

    Ok(output)
} 
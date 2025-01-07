<<<<<<< HEAD
# RustoHebvera

מערכת תרגום דו-כיוונית מתקדמת בין עברית לרוסית עבור מסמכים טכניים.

![Build Status](https://github.com/ofer2300/RustoHebvera/workflows/Rust%20CI%2FCD/badge.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## תכונות עיקריות

- תרגום מונחים טכניים
- תמיכה במסמכי CAD
- ניהול מסמכים ותבניות
- אבטחה מתקדמת
- ממשק משתמש נוח

## דרישות מערכת

- Rust 1.70.0 ומעלה
- Windows 10/11 או Linux
- 4GB RAM מינימום
- 1GB שטח דיסק פנוי

## התקנה

1. התקנת Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. התקנת הספרייה:
```bash
cargo install rusto_hebru
```

## שימוש

```rust
use rusto_hebru::Translator;

let translator = Translator::new();
let translated = translator.translate("שלום עולם", "he", "ru").await?;
```

## ריעוד API

ראו את [התיעוד המלא](docs/API.md) לפרטים נוספים.

## תרומה לפרויקט

אנא קראו את [מדריך התרומה](CONTRIBUTING.md) לפני שאתם מתחילים.

## אבטחה

ראו את [מדיניות האבטחה](SECURITY.md) שלנו.

## רישיון

MIT © RustoHebru Team

## תודות

- תודה לצוות המפתחים והתורמים
- תודה למשתמשים על המשוב והתמיכה
- תודה לקהילת הקוד הפתוח 
=======
# RustoHebvera
>>>>>>> 555c88180dd41dd1980a01115d8ccc321cfd07c2

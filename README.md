# RustoHebvera

מערכת תרגום דו-כיוונית מתקדמת בין עברית לרוסית עבור מסמכים טכניים.

## תכונות עיקריות

- תרגום מונחים טכניים
- תמיכה במסמכי CAD
- ניהול מסמכים ותבניות
- אבטחה מתקדמת
- ממשק משתמש נוח

## התקנה

```bash
cargo install rusto_hebru
```

## שימוש

```rust
use rusto_hebru::Translator;

let translator = Translator::new();
let translated = translator.translate("שלום עולם", "he", "ru").await?;
```

## רישיון

MIT © RustoHebru Team 
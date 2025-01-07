#!/bin/bash

# התקנת CLI של Railway
curl -fsSL https://railway.app/install.sh | sh

# התחברות ל-Railway
railway login

# יצירת פרויקט חדש
railway init

# העלאת האפליקציה
railway up

# הצגת כתובת הגישה
echo "האפליקציה זמינה בכתובת:"
railway show 
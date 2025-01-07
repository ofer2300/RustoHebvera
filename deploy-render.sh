#!/bin/bash

# התקנת CLI של Render
curl -o render https://render.com/download-cli/latest/render-linux-amd64
chmod +x render

# התחברות ל-Render
./render login

# יצירת שירותים
./render create

# הצגת כתובת הגישה
echo "האפליקציה תהיה זמינה בכתובת:"
./render services list | grep rustohebru | awk '{print $4}' 
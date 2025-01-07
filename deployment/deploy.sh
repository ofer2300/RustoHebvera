#!/bin/bash

# הגדרת משתני סביבה
export DOCKER_REGISTRY="registry.digitalocean.com/rustohebru"
export STACK_NAME="rustohebru"
export DOMAIN="rustohebru.com"

# בדיקת תלויות
command -v docker >/dev/null 2>&1 || { echo "נדרש docker" >&2; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "נדרש docker-compose" >&2; exit 1; }

# יצירת swarm אם לא קיים
if ! docker info | grep -q "Swarm: active"; then
    echo "יצירת swarm..."
    docker swarm init
fi

# בניית והעלאת images
echo "בניית והעלאת images..."
docker-compose -f deployment/docker-compose.yml build
docker-compose -f deployment/docker-compose.yml push

# פריסת השירותים
echo "פריסת השירותים..."
docker stack deploy -c deployment/docker-compose.yml $STACK_NAME

# המתנה להשלמת הפריסה
echo "ממתין להשלמת הפריסה..."
sleep 30

# בדיקת סטטוס השירותים
echo "בדיקת סטטוס השירותים..."
docker stack services $STACK_NAME

# הצגת כתובות הגישה
echo "
האפליקציה זמינה בכתובות הבאות:
- אפליקציה: https://$DOMAIN
- ניטור: https://monitor.$DOMAIN
- גרפנה: https://grafana.$DOMAIN
"

# בדיקת בריאות
echo "בדיקת בריאות השירותים..."
curl -f https://$DOMAIN/health || echo "שגיאה בבדיקת בריאות" 
# Быстрый старт

## Требования

- Docker и Docker Compose установлены
- Минимум 4GB свободной RAM
- Порты 8080, 8081, 5432, 6379 свободны

## Запуск системы

### 1. Клонирование и переход в директорию

```bash
cd he-path-of-the-samurai
```

### 2. Опционально: настройка переменных окружения

Создайте файл `.env` в корне проекта (опционально, есть значения по умолчанию):

```bash
# PostgreSQL
DATABASE_URL=postgres://monouser:monopass@db:5432/monolith

# Redis
REDIS_URL=redis://redis:6379

# NASA API (опционально, для расширенного функционала)
NASA_API_KEY=your_nasa_api_key_here
NASA_API_URL=https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json

# Интервалы обновления (в секундах)
FETCH_EVERY_SECONDS=600      # OSDR: 10 минут
ISS_EVERY_SECONDS=120        # ISS: 2 минуты
APOD_EVERY_SECONDS=43200     # APOD: 12 часов
NEO_EVERY_SECONDS=7200       # NEO: 2 часа
DONKI_EVERY_SECONDS=3600     # DONKI: 1 час
SPACEX_EVERY_SECONDS=3600    # SpaceX: 1 час

# HTTP настройки
HTTP_TIMEOUT_SECONDS=30
HTTP_RETRIES=3
RATE_LIMIT_PER_MINUTE=60

# Pascal Legacy
PAS_LEGACY_PERIOD=300        # Генерация CSV каждые 5 минут
```

### 3. Запуск всех сервисов

```bash
docker-compose up -d --build
```

Эта команда:
- Соберет все Docker образы с нуля
- Запустит все сервисы в фоновом режиме
- Подождет готовности зависимостей (health checks)

### 4. Проверка статуса

```bash
# Проверить статус всех контейнеров
docker-compose ps

# Посмотреть логи
docker-compose logs -f

# Логи конкретного сервиса
docker-compose logs -f rust_iss
docker-compose logs -f php_web
docker-compose logs -f pascal_legacy
```

### 5. Доступ к приложению

После запуска (обычно 1-2 минуты на первую сборку):

- **Веб-интерфейс**: http://localhost:8080
  - Dashboard: http://localhost:8080/dashboard
  - ISS: http://localhost:8080/iss
  - OSDR: http://localhost:8080/osdr

- **Rust API**: http://localhost:8081
  - Health check: http://localhost:8081/health
  - ISS последние данные: http://localhost:8081/last
  - OSDR список: http://localhost:8081/osdr/list

- **PostgreSQL**: localhost:5432
  - База: `monolith`
  - Пользователь: `monouser`
  - Пароль: `monopass`

- **Redis**: localhost:6379

## Остановка системы

```bash
# Остановить все сервисы
docker-compose stop

# Остановить и удалить контейнеры
docker-compose down

# Остановить и удалить контейнеры + volumes (удалит все данные!)
docker-compose down -v
```

## Пересборка после изменений

```bash
# Пересобрать конкретный сервис
docker-compose build rust_iss
docker-compose up -d rust_iss

# Пересобрать все
docker-compose build --no-cache
docker-compose up -d
```

## Проверка работоспособности

### 1. Проверка Rust-сервиса

```bash
curl http://localhost:8081/health
# Должен вернуть: {"status":"ok","now":"2024-..."}
```

### 2. Проверка веб-интерфейса

Откройте в браузере: http://localhost:8080/dashboard

Должны отображаться:
- Карточки с метриками МКС
- Карта с позицией МКС
- JWST галерея
- Астрономические события

### 3. Проверка базы данных

```bash
# Подключиться к PostgreSQL
docker exec -it iss_db psql -U monouser -d monolith

# Проверить таблицы
\dt

# Посмотреть данные ISS
SELECT * FROM iss_fetch_log ORDER BY id DESC LIMIT 5;

# Посмотреть OSDR данные
SELECT COUNT(*) FROM osdr_items;

# Выйти
\q
```

### 4. Проверка Redis

```bash
# Подключиться к Redis
docker exec -it redis_cache redis-cli

# Проверить ключи rate limiting
KEYS rate_limit:*

# Проверить ping
PING
# Должен вернуть: PONG

# Выйти
exit
```

### 5. Проверка Pascal-Legacy

```bash
# Проверить логи
docker logs pascal_legacy

# Проверить CSV файлы
docker exec pascal_legacy ls -lh /data/csv/

# Проверить XLSX файлы (должны появиться автоматически)
docker exec pascal_legacy ls -lh /data/csv/*.xlsx
```

## Устранение проблем

### Проблема: Контейнеры не запускаются

```bash
# Проверить логи
docker-compose logs

# Проверить, заняты ли порты
lsof -i :8080
lsof -i :8081
lsof -i :5432
lsof -i :6379

# Остановить конфликтующие процессы или изменить порты в docker-compose.yml
```

### Проблема: Rust-сервис не подключается к БД

```bash
# Проверить, что БД запущена
docker-compose ps db

# Проверить логи БД
docker-compose logs db

# Проверить переменные окружения
docker exec rust_iss env | grep DATABASE_URL
```

### Проблема: Laravel показывает ошибки

```bash
# Проверить логи PHP
docker-compose logs php

# Проверить права доступа
docker exec php_web ls -la /var/www/html

# Пересоздать контейнер
docker-compose restart php
```

### Проблема: Нет данных на дашборде

1. Подождите 2-3 минуты (фоновые задачи запускаются с интервалами)
2. Проверьте логи Rust-сервиса: `docker-compose logs rust_iss`
3. Проверьте, что внешние API доступны (может быть проблема с сетью)
4. Вручную запустите синхронизацию: `curl http://localhost:8081/osdr/sync`

## Полезные команды

```bash
# Посмотреть использование ресурсов
docker stats

# Очистить неиспользуемые образы
docker system prune -a

# Посмотреть все volumes
docker volume ls

# Удалить конкретный volume
docker volume rm he-path-of-the-samurai_pgdata

# Войти в контейнер
docker exec -it rust_iss sh
docker exec -it php_web bash
docker exec -it iss_db psql -U monouser -d monolith
```

## Структура проекта

```
he-path-of-the-samurai/
├── docker-compose.yml          # Конфигурация всех сервисов
├── db/
│   └── init.sql                # Инициализация БД
├── services/
│   ├── rust-iss/               # Rust-сервис
│   │   ├── src/                # Исходный код
│   │   ├── Cargo.toml          # Зависимости
│   │   └── Dockerfile          # Образ Rust-сервиса
│   ├── php-web/                # Laravel приложение
│   │   ├── laravel-patches/    # Патчи Laravel
│   │   └── Dockerfile          # Образ PHP
│   └── pascal-legacy/          # Легаси-утилита
│       ├── legacy.pas           # Pascal код
│       ├── csv_to_xlsx.py      # Конвертер CSV→XLSX
│       └── Dockerfile           # Образ Pascal
└── README.md                    # Документация
```

## Следующие шаги

После успешного запуска:

1. Изучите документацию:
   - `ARCHITECTURE.md` - Архитектура системы
   - `EXPERTS_REVIEW.md` - Экспертное мнение
   - `TESTING.md` - Руководство по тестированию

2. Настройте мониторинг (опционально):
   - Добавьте Prometheus метрики
   - Настройте логирование в файлы

3. Настройте production окружение:
   - Измените пароли БД
   - Настройте SSL/TLS
   - Добавьте backup стратегию

## Поддержка

При возникновении проблем:
1. Проверьте логи: `docker-compose logs`
2. Изучите документацию в `ARCHITECTURE.md`
3. Проверьте health checks: `docker-compose ps`


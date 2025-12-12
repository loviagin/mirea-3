# Архитектура системы "Кассиопея"

## Обзор системы

Система представляет собой распределенный монолит для сбора и визуализации космических данных, состоящий из следующих компонентов:

- **rust_iss** - Rust-сервис для опроса внешних API и хранения данных
- **php_web** - Laravel веб-приложение с дашбордами
- **iss_db** - PostgreSQL база данных
- **pascal_legacy** - Легаси-утилита для генерации CSV/XLSX
- **nginx** - Reverse proxy
- **redis** - Кэш и rate-limiting

## Архитектура Rust-сервиса (rust_iss)

### Слоистая архитектура (Clean Architecture)

```
src/
├── main.rs          # Точка входа, инициализация
├── config.rs        # Конфигурация из env
├── domain.rs        # Доменные модели и ошибки
├── repo.rs          # Репозитории для работы с БД
├── clients.rs       # HTTP-клиенты для внешних API
├── services.rs      # Бизнес-логика
├── handlers.rs      # HTTP-обработчики (Axum)
├── routes.rs        # Маршрутизация
├── state.rs         # AppState для DI
├── validation.rs    # Валидация данных
└── middleware.rs    # Rate-limiting middleware
```

### Dependency Injection через AppState

```rust
pub struct AppState {
    pub pool: PgPool,                    // PostgreSQL connection pool
    pub iss_client: Arc<IssClient>,      // ISS API client
    pub nasa_client: Arc<NasaClient>,    // NASA API client
    pub spacex_client: Arc<SpaceXClient>, // SpaceX API client
    pub config: Config,                  // Конфигурация
}
```

### Обработка ошибок

Все ошибки представлены через `ApiError` enum с автоматической конвертацией в HTTP-ответы:

```rust
pub enum ApiError {
    Database(sqlx::Error),
    Http(reqwest::Error),
    Validation(String),
    NotFound,
    Internal(String),
}
```

### Типы данных для TIMESTAMPTZ

Используется `chrono::DateTime<Utc>` для работы с PostgreSQL `TIMESTAMPTZ`:

```rust
pub struct IssFetchLog {
    pub fetched_at: DateTime<Utc>,  // TIMESTAMPTZ
    // ...
}
```

### Upsert vs INSERT

**Upsert (ON CONFLICT DO UPDATE)** используется для OSDR данных:
- Предотвращает дубликаты по `dataset_id`
- Обновляет существующие записи
- Эффективнее чем проверка + INSERT

**INSERT** используется для:
- ISS логов (каждая запись уникальна)
- Space cache (исторические данные)

### Защита от наложения задач (Mutex/Advisory Lock)

Используются PostgreSQL Advisory Locks для предотвращения параллельного выполнения фоновых задач:

```rust
let lock_key: i64 = 1001; // Уникальный ключ для каждой задачи
if let Ok(Some(_)) = sqlx::query_scalar("SELECT pg_try_advisory_lock($1)")
    .bind(lock_key)
    .fetch_optional(&pool)
    .await
{
    // Выполнение задачи
    // ...
    sqlx::query("SELECT pg_advisory_unlock($1)").execute(&pool).await?;
}
```

### Rate Limiting

Реализован через Redis с использованием IP-адреса клиента:

- Лимит: 60 запросов в минуту (настраивается через `RATE_LIMIT_PER_MINUTE`)
- Использует ключи вида `rate_limit:{ip}`
- TTL: 60 секунд

### HTTP-клиенты с retry

Все внешние API-клиенты имеют:
- Настраиваемый timeout (`HTTP_TIMEOUT_SECONDS`)
- Retry механизм (`HTTP_RETRIES`)
- User-Agent для идентификации
- Экспоненциальная задержка между попытками

## Архитектура Laravel (php_web)

### Разделение на контексты (страницы)

- `/dashboard` - Главный дашборд с ISS картой и JWST галереей
- `/iss` - Детальная информация о МКС
- `/osdr` - Таблица OSDR данных с фильтрацией

### Фильтрация и поиск

Реализовано на клиентской стороне (JavaScript):
- Поиск по ключевым словам
- Сортировка по любому столбцу (возрастание/убывание)
- Подсветка найденных результатов
- Работа с датами и числами

### Анимации и CSS

- Fade-in анимации при загрузке
- Hover-эффекты на карточках
- Современный дизайн с Bootstrap 5
- Адаптивная верстка

## Pascal-Legacy

### Генерация CSV с правильными типами

1. **TIMESTAMP**: ISO 8601 формат (`yyyy-mm-ddTHH:MM:SSZ`)
2. **BOOLEAN**: `ИСТИНА` / `ЛОЖЬ` (русский формат)
3. **Числа**: Числовой формат без кавычек
4. **Строки**: Текст в кавычках

### Конвертация в XLSX

Автоматическая конвертация через Python-скрипт `csv_to_xlsx.py`:
- Определение типов данных автоматически
- Форматирование дат в Excel
- Boolean значения как true/false
- Автоматическая ширина столбцов
- Стилизованные заголовки

## База данных (PostgreSQL)

### Основные таблицы

1. **iss_fetch_log** - Логи запросов ISS API
2. **osdr_items** - Данные NASA OSDR с уникальным индексом по `dataset_id`
3. **space_cache** - Кэш космических данных (APOD, NEO, DONKI, SpaceX)
4. **telemetry_legacy** - Данные из Pascal-Legacy модуля

### Индексы

- `ux_osdr_dataset_id` - Уникальный индекс для upsert
- `ix_space_cache_source` - Индекс для быстрого поиска по источнику

## Docker Compose

### Сервисы

- `db` - PostgreSQL 16
- `redis` - Redis 7 для кэширования и rate-limiting
- `rust_iss` - Rust-сервис (порт 8081)
- `php` - Laravel приложение
- `nginx` - Reverse proxy (порт 8080)
- `pascal_legacy` - Легаси-утилита

### Volumes

- `pgdata` - Данные PostgreSQL
- `appdata` - Данные Laravel
- `csvdata` - CSV/XLSX файлы
- `redisdata` - Данные Redis

## Внешние API

### Используемые API

1. **WhereTheISS** - Позиция МКС
2. **NASA OSDR** - Биологические данные
3. **NASA APOD** - Астрономическая картинка дня
4. **NASA NEO** - Близкие к Земле объекты
5. **NASA DONKI** - Космическая погода (FLR, CME)
6. **SpaceX API** - Информация о запусках
7. **JWST API** - Изображения James Webb Telescope
8. **AstronomyAPI** - Астрономические события

### Защита от бана

- Таймауты для всех запросов
- Retry механизм с экспоненциальной задержкой
- User-Agent идентификация
- Rate-limiting на стороне сервиса

## Производительность

### Оптимизации

1. **Connection Pooling** - Пул соединений PostgreSQL (max 5)
2. **Advisory Locks** - Предотвращение дублирования задач
3. **Redis Caching** - Кэширование для rate-limiting
4. **Upsert** - Эффективное обновление данных
5. **Индексы БД** - Быстрый поиск по источникам
6. **Асинхронность** - Все операции неблокирующие (Tokio)

### Паттерны производительности

- **Repository Pattern** - Изоляция логики доступа к данным
- **Service Layer** - Разделение бизнес-логики
- **Dependency Injection** - Тестируемость и гибкость
- **Error Handling** - Централизованная обработка ошибок

## Безопасность

- Rate-limiting для защиты от DDoS
- Валидация входных данных
- SQL injection защита через параметризованные запросы
- XSS защита в Laravel Blade (автоматический экранирование)


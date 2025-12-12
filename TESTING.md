# Тестирование системы "Кассиопея"

## Rust-сервис (rust_iss)

### Модульные тесты

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_haversine_calculation() {
        // Тест расчета расстояния между точками
        let km = haversine_km(55.7558, 37.6176, 55.7520, 37.6156);
        assert!(km > 0.0 && km < 1.0);
    }
    
    #[test]
    fn test_extract_string() {
        let json = serde_json::json!({"title": "Test", "name": "Alternative"});
        let result = extract_string(&json, &["title", "name"]);
        assert_eq!(result, Some("Test".to_string()));
    }
}
```

### Интеграционные тесты

```rust
#[tokio::test]
async fn test_iss_repo_create() {
    let pool = setup_test_db().await;
    let id = IssRepo::create(&pool, "http://test", serde_json::json!({})).await.unwrap();
    assert!(id > 0);
}
```

### Тесты API endpoints

Используйте `axum-test` для тестирования HTTP endpoints:

```rust
use axum_test::TestServer;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_router().with_state(test_state());
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), 200);
}
```

## Laravel (php_web)

### Unit тесты

```php
// tests/Unit/OsdrControllerTest.php
class OsdrControllerTest extends TestCase
{
    public function test_flatten_osdr()
    {
        $controller = new OsdrController();
        $items = [
            ['id' => 1, 'raw' => ['OSD-1' => ['title' => 'Test']]]
        ];
        $result = $controller->flattenOsdr($items);
        $this->assertCount(1, $result);
        $this->assertEquals('Test', $result[0]['title']);
    }
}
```

### Feature тесты

```php
// tests/Feature/DashboardTest.php
class DashboardTest extends TestCase
{
    public function test_dashboard_loads()
    {
        $response = $this->get('/dashboard');
        $response->assertStatus(200);
        $response->assertSee('МКС');
    }
}
```

## Интеграционные тесты

### Тест полного потока данных

1. Rust-сервис получает данные из внешнего API
2. Сохраняет в PostgreSQL
3. Laravel запрашивает данные через proxy
4. Отображает на дашборде

### Тест rate-limiting

```bash
# Отправка 100 запросов подряд
for i in {1..100}; do
  curl http://localhost:8081/health
done
# Должен вернуть 429 после 60 запросов
```

### Тест advisory locks

Проверка, что фоновые задачи не выполняются параллельно:

```sql
-- В одной сессии
SELECT pg_try_advisory_lock(1001); -- Вернет true

-- В другой сессии
SELECT pg_try_advisory_lock(1001); -- Вернет false (занято)
```

## Нагрузочное тестирование

### Apache Bench

```bash
ab -n 1000 -c 10 http://localhost:8080/dashboard
```

### wrk

```bash
wrk -t4 -c100 -d30s http://localhost:8081/health
```

## Тестирование Pascal-Legacy

### Тест генерации CSV

```bash
docker exec pascal_legacy /app/legacy
# Проверка файла
cat /data/csv/telemetry_*.csv
```

### Тест конвертации в XLSX

```bash
python3 csv_to_xlsx.py test.csv test.xlsx
# Проверка типов данных в Excel
```

## Проверка типов данных

### TIMESTAMPTZ

```sql
SELECT 
    fetched_at,
    pg_typeof(fetched_at) as type
FROM iss_fetch_log 
LIMIT 1;
-- Должен быть: timestamp with time zone
```

### Boolean в CSV

Проверка формата `ИСТИНА`/`ЛОЖЬ`:

```bash
grep -E "ИСТИНА|ЛОЖЬ" /data/csv/*.csv
```

## Мониторинг

### Логи

```bash
# Rust-сервис
docker logs rust_iss

# Laravel
docker logs php_web

# Pascal-Legacy
docker logs pascal_legacy
```

### Метрики PostgreSQL

```sql
-- Активные соединения
SELECT count(*) FROM pg_stat_activity;

-- Размер таблиц
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Redis метрики

```bash
redis-cli INFO stats
redis-cli KEYS "rate_limit:*"
```


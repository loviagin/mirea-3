#!/usr/bin/env bash
set -e
cd /app
echo "[pascal] compiling legacy.pas"
fpc -O2 -S2 legacy.pas
echo "[pascal] running legacy CSV generator and importer"

# Фоновая конвертация CSV в XLSX
CSV_DIR="${CSV_OUT_DIR:-/data/csv}"
echo "[pascal] Starting CSV to XLSX converter watcher in $CSV_DIR"

# Функция конвертации всех CSV в директории
convert_all_csv() {
    for csv_file in "$CSV_DIR"/*.csv; do
        if [ -f "$csv_file" ]; then
            xlsx_file="${csv_file%.csv}.xlsx"
            if [ ! -f "$xlsx_file" ] || [ "$csv_file" -nt "$xlsx_file" ]; then
                echo "[pascal] Converting $csv_file to XLSX..."
                python3 /app/csv_to_xlsx.py "$csv_file" "$xlsx_file" || true
            fi
        fi
    done
}

# Запуск конвертера в фоне
(
    while true; do
        sleep 10
        convert_all_csv
    done
) &

# Запуск основного процесса
./legacy

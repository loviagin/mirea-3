program LegacyCSV;

{$mode objfpc}{$H+}

uses
  SysUtils, DateUtils, Unix, Math;

function GetEnvDef(const name, def: string): string;
var v: string;
begin
  v := GetEnvironmentVariable(name);
  if v = '' then Exit(def) else Exit(v);
end;

function RandFloat(minV, maxV: Double): Double;
begin
  Result := minV + Random * (maxV - minV);
end;

procedure GenerateAndCopy();
var
  outDir, fn, fullpath, pghost, pgport, pguser, pgpass, pgdb, copyCmd: string;
  f: TextFile;
  ts: string;
  timestamp: TDateTime;
  voltage, temp: Double;
  is_active: Boolean;
  status_text: string;
begin
  outDir := GetEnvDef('CSV_OUT_DIR', '/data/csv');
  ts := FormatDateTime('yyyymmdd_hhnnss', Now);
  fn := 'telemetry_' + ts + '.csv';
  fullpath := IncludeTrailingPathDelimiter(outDir) + fn;

  // Генерация данных с правильными типами
  timestamp := Now;
  voltage := RandFloat(3.2, 12.6);
  temp := RandFloat(-50.0, 80.0);
  is_active := Random(2) = 1; // Boolean: ИСТИНА или ЛОЖЬ
  status_text := 'operational';

  // write CSV с правильными типами:
  // - timestamp: ISO 8601 формат
  // - boolean: ИСТИНА/ЛОЖЬ
  // - числа: числовой формат
  // - строки: текст
  AssignFile(f, fullpath);
  Rewrite(f);
  // Заголовок
  Writeln(f, 'recorded_at,voltage,temp,is_active,status_text,source_file');
  // Данные с правильными типами
  if is_active then
    Writeln(f, 
      FormatDateTime('yyyy-mm-dd"T"hh:nn:ss"Z"', timestamp) + ',' +  // TIMESTAMP в ISO 8601
      FormatFloat('0.00', voltage) + ',' +                          // Число
      FormatFloat('0.00', temp) + ',' +                             // Число
      'ИСТИНА' + ',' +                                              // Boolean: ИСТИНА
      '"' + status_text + '"' + ',' +                                // Строка в кавычках
      '"' + fn + '"'                                                  // Строка в кавычках
    )
  else
    Writeln(f, 
      FormatDateTime('yyyy-mm-dd"T"hh:nn:ss"Z"', timestamp) + ',' +  // TIMESTAMP в ISO 8601
      FormatFloat('0.00', voltage) + ',' +                          // Число
      FormatFloat('0.00', temp) + ',' +                             // Число
      'ЛОЖЬ' + ',' +                                               // Boolean: ЛОЖЬ
      '"' + status_text + '"' + ',' +                                // Строка в кавычках
      '"' + fn + '"'                                                  // Строка в кавычках
    );
  CloseFile(f);

  // COPY into Postgres
  pghost := GetEnvDef('PGHOST', 'db');
  pgport := GetEnvDef('PGPORT', '5432');
  pguser := GetEnvDef('PGUSER', 'monouser');
  pgpass := GetEnvDef('PGPASSWORD', 'monopass');
  pgdb   := GetEnvDef('PGDATABASE', 'monolith');

  // Use psql with COPY FROM PROGRAM for simplicity
  // Here we call psql reading from file
  // Устанавливаем PGPASSWORD через переменную окружения в команде
  copyCmd := 'PGPASSWORD=' + pgpass + ' psql "host=' + pghost + ' port=' + pgport + ' user=' + pguser + ' dbname=' + pgdb + '" ' +
             '-c "\copy telemetry_legacy(recorded_at, voltage, temp, is_active, status_text, source_file) FROM ''' + fullpath + ''' WITH (FORMAT csv, HEADER true)"';
  // Execute
  fpSystem(copyCmd);
end;

var period: Integer;
begin
  Randomize;
  period := StrToIntDef(GetEnvDef('GEN_PERIOD_SEC', '300'), 300);
  while True do
  begin
    try
      GenerateAndCopy();
    except
      on E: Exception do
        WriteLn('Legacy error: ', E.Message);
    end;
    Sleep(period * 1000);
  end;
end.

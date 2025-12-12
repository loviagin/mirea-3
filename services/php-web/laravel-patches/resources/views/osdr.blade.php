@extends('layouts.app')

@section('content')
<style>
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .fade-in {
    animation: fadeIn 0.5s ease-in;
  }
  .table-sortable th {
    cursor: pointer;
    user-select: none;
    position: relative;
  }
  .table-sortable th:hover {
    background-color: #f8f9fa;
  }
  .sort-icon {
    display: inline-block;
    margin-left: 5px;
    opacity: 0.5;
    font-size: 0.8em;
  }
  .sort-icon.active {
    opacity: 1;
  }
  .search-highlight {
    background-color: #fff3cd;
    padding: 2px 4px;
    border-radius: 3px;
  }
</style>

<div class="container py-4 fade-in">
  <div class="d-flex justify-content-between align-items-center mb-4">
    <h3 class="mb-0">NASA OSDR</h3>
    <div class="small text-muted">Источник: <code>{{ $src }}</code></div>
  </div>

  {{-- Фильтры и поиск --}}
  <div class="card mb-4 shadow-sm">
    <div class="card-body">
      <div class="row g-3">
        <div class="col-md-4">
          <label class="form-label">Поиск по ключевым словам</label>
          <input type="text" id="searchInput" class="form-control" placeholder="Введите текст для поиска...">
        </div>
        <div class="col-md-3">
          <label class="form-label">Сортировка по столбцу</label>
          <select id="sortColumn" class="form-select">
            <option value="id">ID</option>
            <option value="dataset_id">Dataset ID</option>
            <option value="title">Title</option>
            <option value="updated_at" selected>Updated At</option>
            <option value="inserted_at">Inserted At</option>
          </select>
        </div>
        <div class="col-md-3">
          <label class="form-label">Направление сортировки</label>
          <select id="sortDirection" class="form-select">
            <option value="asc">По возрастанию</option>
            <option value="desc" selected>По убыванию</option>
          </select>
        </div>
        <div class="col-md-2 d-flex align-items-end">
          <button id="resetFilters" class="btn btn-outline-secondary w-100">Сбросить</button>
        </div>
      </div>
    </div>
  </div>

  <div class="table-responsive">
    <table class="table table-sm table-striped align-middle table-sortable" id="osdrTable">
      <thead class="table-light">
        <tr>
          <th data-column="id"># <span class="sort-icon">⇅</span></th>
          <th data-column="dataset_id">dataset_id <span class="sort-icon">⇅</span></th>
          <th data-column="title">title <span class="sort-icon">⇅</span></th>
          <th>REST_URL</th>
          <th data-column="updated_at">updated_at <span class="sort-icon">⇅</span></th>
          <th data-column="inserted_at">inserted_at <span class="sort-icon">⇅</span></th>
          <th>raw</th>
        </tr>
      </thead>
      <tbody id="tableBody">
      @forelse($items as $row)
        <tr data-row='@json($row)'>
          <td>{{ $row['id'] }}</td>
          <td>{{ $row['dataset_id'] ?? '—' }}</td>
          <td style="max-width:420px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
            {{ $row['title'] ?? '—' }}
          </td>
          <td>
            @if(!empty($row['rest_url']))
              <a href="{{ $row['rest_url'] }}" target="_blank" rel="noopener">открыть</a>
            @else — @endif
          </td>
          <td>{{ $row['updated_at'] ?? '—' }}</td>
          <td>{{ $row['inserted_at'] ?? '—' }}</td>
          <td>
            <button class="btn btn-outline-secondary btn-sm" data-bs-toggle="collapse" data-bs-target="#raw-{{ $row['id'] }}-{{ md5($row['dataset_id'] ?? (string)$row['id']) }}">JSON</button>
          </td>
        </tr>
        <tr class="collapse" id="raw-{{ $row['id'] }}-{{ md5($row['dataset_id'] ?? (string)$row['id']) }}">
          <td colspan="7">
            <pre class="mb-0" style="max-height:260px;overflow:auto">{{ json_encode($row['raw'] ?? [], JSON_PRETTY_PRINT|JSON_UNESCAPED_SLASHES|JSON_UNESCAPED_UNICODE) }}</pre>
          </td>
        </tr>
      @empty
        <tr><td colspan="7" class="text-center text-muted">нет данных</td></tr>
      @endforelse
      </tbody>
    </table>
  </div>
</div>

<script>
document.addEventListener('DOMContentLoaded', function() {
  const tableBody = document.getElementById('tableBody');
  const searchInput = document.getElementById('searchInput');
  const sortColumn = document.getElementById('sortColumn');
  const sortDirection = document.getElementById('sortDirection');
  const resetBtn = document.getElementById('resetFilters');
  const sortableHeaders = document.querySelectorAll('.table-sortable th[data-column]');
  
  let allRows = Array.from(tableBody.querySelectorAll('tr[data-row]'));
  let currentSortColumn = 'updated_at';
  let currentSortDirection = 'desc';
  
  // Сохранение исходных данных
  const originalRows = allRows.map(row => ({
    element: row,
    data: JSON.parse(row.getAttribute('data-row')),
    nextElement: row.nextElementSibling
  }));
  
  function highlightText(text, search) {
    if (!search) return text;
    const regex = new RegExp(`(${search})`, 'gi');
    return text.replace(regex, '<span class="search-highlight">$1</span>');
  }
  
  function filterAndSort() {
    const search = searchInput.value.toLowerCase().trim();
    const sortCol = sortColumn.value;
    const sortDir = sortDirection.value;
    
    // Фильтрация
    let filtered = originalRows.filter(item => {
      if (!search) return true;
      const data = item.data;
      const searchable = [
        String(data.id || ''),
        String(data.dataset_id || ''),
        String(data.title || ''),
        String(data.status || ''),
        String(data.updated_at || ''),
        String(data.inserted_at || '')
      ].join(' ').toLowerCase();
      return searchable.includes(search);
    });
    
    // Сортировка
    filtered.sort((a, b) => {
      let aVal = a.data[sortCol];
      let bVal = b.data[sortCol];
      
      // Обработка дат
      if (sortCol.includes('_at') || sortCol.includes('at')) {
        aVal = aVal ? new Date(aVal).getTime() : 0;
        bVal = bVal ? new Date(bVal).getTime() : 0;
      }
      
      // Обработка null/undefined
      if (aVal == null) aVal = '';
      if (bVal == null) bVal = '';
      
      if (typeof aVal === 'string') aVal = aVal.toLowerCase();
      if (typeof bVal === 'string') bVal = bVal.toLowerCase();
      
      let result = 0;
      if (aVal < bVal) result = -1;
      else if (aVal > bVal) result = 1;
      
      return sortDir === 'asc' ? result : -result;
    });
    
    // Обновление таблицы
    tableBody.innerHTML = '';
    filtered.forEach(item => {
      const row = item.element.cloneNode(true);
      const cells = row.querySelectorAll('td');
      
      // Подсветка поиска
      if (search) {
        cells.forEach(cell => {
          const text = cell.textContent;
          cell.innerHTML = highlightText(text, search);
        });
      }
      
      tableBody.appendChild(row);
      if (item.nextElement && item.nextElement.classList.contains('collapse')) {
        tableBody.appendChild(item.nextElement.cloneNode(true));
      }
    });
    
    // Обновление иконок сортировки
    sortableHeaders.forEach(header => {
      const icon = header.querySelector('.sort-icon');
      if (header.dataset.column === sortCol) {
        icon.textContent = sortDir === 'asc' ? '↑' : '↓';
        icon.classList.add('active');
      } else {
        icon.textContent = '⇅';
        icon.classList.remove('active');
      }
    });
  }
  
  // Обработчики событий
  searchInput.addEventListener('input', filterAndSort);
  sortColumn.addEventListener('change', filterAndSort);
  sortDirection.addEventListener('change', filterAndSort);
  
  sortableHeaders.forEach(header => {
    header.addEventListener('click', () => {
      const column = header.dataset.column;
      if (currentSortColumn === column) {
        currentSortDirection = currentSortDirection === 'asc' ? 'desc' : 'asc';
      } else {
        currentSortColumn = column;
        currentSortDirection = 'desc';
      }
      sortColumn.value = currentSortColumn;
      sortDirection.value = currentSortDirection;
      filterAndSort();
    });
  });
  
  resetBtn.addEventListener('click', () => {
    searchInput.value = '';
    sortColumn.value = 'updated_at';
    sortDirection.value = 'desc';
    currentSortColumn = 'updated_at';
    currentSortDirection = 'desc';
    filterAndSort();
  });
  
  // Инициализация
  filterAndSort();
});
</script>
@endsection

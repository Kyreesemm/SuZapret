(() => {
  'use strict';

  const strategies = [
    'general', 'general_alt', 'general_alt2', 'general_alt3', 'general_alt4', 'general_alt5',
    'general_alt6', 'general_alt7', 'general_alt8', 'general_alt9', 'general_alt10', 'general_alt11',
    'general_alt12', 'general_alt13', 'general_exp', 'general_fake_tls_auto', 'general_fake_tls_auto_alt',
    'general_fake_tls_auto_alt2', 'general_fake_tls_auto_alt3', 'general_nix1',
    'general_simple_fake_alt', 'general_simple_fake_alt2', 'general_simple_fake_custom', 'general_simple_fake_flowseal',
    'minecraft_hypixel_etc'
  ];
  const state = { running: false, selectedStrategy: strategies[0] };
  const $ = (selector) => document.querySelector(selector);
  const strategySelect = $('#strategySelect');
  const appWindow = $('#appWindow');
  const terminal = $('#terminalLog');

  function readableName(id) {
    return id.replaceAll('_', ' ').replace(/\b\w/g, (letter) => letter.toUpperCase());
  }

  function populateStrategySelect() {
    strategySelect.innerHTML = strategies.map((id) => `<option value="${id}">${readableName(id)}</option>`).join('');
    $('#strategyCountBadge').textContent = `${strategies.length} вариантов`;
    strategySelect.value = state.selectedStrategy;
  }

  function addLog(message, type = 'log-info') {
    const timestamp = new Date().toLocaleTimeString('ru-RU', { hour12: false });
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.textContent = `[${timestamp}] ${message}`;
    terminal.append(entry);
    terminal.scrollTop = terminal.scrollHeight;
  }

  function setRunning(running) {
    state.running = running;
    appWindow.classList.toggle('is-running', running);
    $('#btnLabel').textContent = running ? 'Остановить SuZapret' : 'Запустить SuZapret';
    $('#statusText').textContent = running ? 'SuZapret работает' : 'SuZapret остановлен';
    addLog(running ? `[CORE] Запущена стратегия: ${state.selectedStrategy}` : '[CORE] SuZapret остановлен', running ? 'log-success' : 'log-warn');
  }

  document.querySelectorAll('.nav-tab').forEach((tab) => tab.addEventListener('click', () => {
    document.querySelectorAll('.nav-tab').forEach((item) => { item.classList.remove('active'); item.setAttribute('aria-selected', 'false'); });
    document.querySelectorAll('.tab-content').forEach((content) => content.classList.remove('active'));
    tab.classList.add('active');
    tab.setAttribute('aria-selected', 'true');
    $(`#${tab.dataset.tab}`).classList.add('active');
  }));

  strategySelect.addEventListener('change', () => {
    state.selectedStrategy = strategySelect.value;
    addLog(`[UI] Выбрана стратегия: ${state.selectedStrategy}`);
  });
  $('#toggleBtn').addEventListener('click', () => setRunning(!state.running));
  $('#clearLogsBtn').addEventListener('click', () => { terminal.innerHTML = '<div class="log-entry log-system">[SYSTEM] Журнал очищен</div>'; });
  $('#gameFilterToggle').addEventListener('change', (event) => { $('#toggleGfTcp').checked = event.target.checked; $('#toggleGfUdp').checked = event.target.checked; });
  populateStrategySelect();
})();

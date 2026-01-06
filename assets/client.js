let ws = null;

// DOM elements
const statusEl = document.getElementById('status');
const chatMessagesEl = document.getElementById('chat-messages');
const systemMessagesEl = document.getElementById('system-messages');
const subscriptionsEl = document.getElementById('subscriptions');
const publishChannelEl = document.getElementById('publishChannel');
const connectBtn = document.getElementById('connectBtn');
const disconnectBtn = document.getElementById('disconnectBtn');
const subscribeBtn = document.getElementById('subscribeBtn');
const refreshBtn = document.getElementById('refreshBtn');
const publishBtn = document.getElementById('publishBtn');

function connect() {
  const clientName = document.getElementById('clientName').value.trim();

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  let url = `${protocol}//${window.location.host}/api/connect`;

  if (clientName) {
    url += `?client_name=${encodeURIComponent(clientName)}`;
  }

  ws = new WebSocket(url);

  ws.onopen = () => {
    setConnected(true);
    addMessage('system', 'system', 'Connected to server');
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      handleServerMessage(data);
    } catch (e) {
      addMessage('system', 'error', `Failed to parse message: ${event.data}`);
    }
  };

  ws.onclose = () => {
    setConnected(false);
    addMessage('system', 'system', 'Disconnected from server');
    updateSubscriptionsUI([]);
  };

  ws.onerror = () => {
    addMessage('system', 'error', 'WebSocket error occurred');
  };
}

function disconnect() {
  if (ws) {
    ws.close();
    ws = null;
  }
}

function setConnected(connected) {
  statusEl.textContent = connected ? 'Connected' : 'Disconnected';
  statusEl.className = connected ? 'connected' : 'disconnected';
  connectBtn.disabled = connected;
  disconnectBtn.disabled = !connected;
  subscribeBtn.disabled = !connected;
  refreshBtn.disabled = !connected;
  document.getElementById('clientName').disabled = connected;
}

function handleServerMessage(data) {
  // Chat message (has sender, channel_name, content, sent_at)
  if (data.sender && data.channel_name && data.content && data.sent_at) {
    const time = new Date(data.sent_at).toLocaleTimeString();
    addMessage('chat', 'incoming', `[${time}] #${data.channel_name} - ${data.sender}: ${data.content}`);
    return;
  }

  // Response/status message
  if (data.status === 'ok') {
    if (data.info) {
      if (data.info.client_name) {
        addMessage('system', 'system', `Connected as: ${data.info.client_name}`);
      }
      if (data.info.channel_name && data.info.total_subscribers !== undefined) {
        addMessage('system', 'system', `Subscribed to #${data.info.channel_name} (${data.info.total_subscribers} subscribers)`);
        refreshChannels();
      }
      if (data.info.channels !== undefined) {
        updateSubscriptionsUI(data.info.channels);
      }
    } else {
      addMessage('system', 'system', 'OK');
    }
  } else if (data.status === 'error') {
    const detail = data.info?.detail || 'Unknown error';
    addMessage('system', 'error', `Error: ${detail}`);
  } else {
    addMessage('system', 'incoming', JSON.stringify(data, null, 2));
  }
}

function subscribe() {
  const channel = document.getElementById('subscribeChannel').value.trim();
  if (!channel) {
    addMessage('system', 'error', 'Please enter a channel name');
    return;
  }

  sendMessage({
    action: 'subscribe',
    params: { channel_name: channel }
  });
  document.getElementById('subscribeChannel').value = '';
}

function unsubscribe(channel) {
  sendMessage({
    action: 'unsubscribe',
    params: { channel_name: channel }
  });
  refreshChannels();
}

function refreshChannels() {
  sendMessage({ action: 'list' });
}

function publish() {
  const channel = publishChannelEl.value;
  const content = document.getElementById('messageContent').value.trim();

  if (!channel) {
    addMessage('system', 'error', 'Please select a channel');
    return;
  }
  if (!content) {
    addMessage('system', 'error', 'Please enter a message');
    return;
  }

  sendMessage({
    action: 'publish',
    params: { channel_name: channel, content: content }
  });
  addMessage('chat', 'outgoing', `[Sent to #${channel}]: ${content}`);
  document.getElementById('messageContent').value = '';
}

function sendMessage(msg) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(msg));
  } else {
    addMessage('system', 'error', 'Not connected to server');
  }
}

function updateSubscriptionsUI(channels) {
  if (!channels || channels.length === 0) {
    subscriptionsEl.innerHTML = '<em style="color: #999;">None</em>';
    publishChannelEl.innerHTML = '<option value="">-- Subscribe first --</option>';
    publishChannelEl.disabled = true;
    publishBtn.disabled = true;
  } else {
    subscriptionsEl.innerHTML = '';
    publishChannelEl.innerHTML = '';

    channels.forEach(channel => {
      const tag = document.createElement('span');
      tag.className = 'subscription-tag';
      tag.innerHTML = `#${channel} <span class="remove" onclick="unsubscribe('${channel}')">&times;</span>`;
      subscriptionsEl.appendChild(tag);

      const option = document.createElement('option');
      option.value = channel;
      option.textContent = `#${channel}`;
      publishChannelEl.appendChild(option);
    });

    publishChannelEl.disabled = false;
    if (ws && ws.readyState === WebSocket.OPEN) {
      publishBtn.disabled = false;
    }
  }
}

function switchTab(tabName) {
  document.querySelectorAll('.tab').forEach(tab => {
    tab.classList.toggle('active', tab.dataset.tab === tabName);
  });
  document.querySelectorAll('.tab-content').forEach(content => {
    content.classList.toggle('active', content.id === `${tabName}-messages`);
  });
}

function addMessage(tab, type, text) {
  const container = tab === 'chat' ? chatMessagesEl : systemMessagesEl;
  const div = document.createElement('div');
  div.className = `msg ${type}`;

  const now = new Date().toLocaleTimeString();
  div.innerHTML = `<div class="meta">${now}</div>${escapeHtml(text)}`;

  container.appendChild(div);
  container.scrollTop = container.scrollHeight;
}

function clearMessages(tab) {
  const container = tab === 'chat' ? chatMessagesEl : systemMessagesEl;
  container.innerHTML = '';
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

document.getElementById('messageContent').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') publish();
});

document.getElementById('subscribeChannel').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') subscribe();
});

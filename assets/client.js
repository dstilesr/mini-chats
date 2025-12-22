let ws = null;
const subscriptions = new Set();

// DOM elements
const statusEl = document.getElementById('status');
const messagesEl = document.getElementById('messages');
const subscriptionsEl = document.getElementById('subscriptions');
const publishChannelEl = document.getElementById('publishChannel');
const connectBtn = document.getElementById('connectBtn');
const disconnectBtn = document.getElementById('disconnectBtn');
const subscribeBtn = document.getElementById('subscribeBtn');
const publishBtn = document.getElementById('publishBtn');

function connect() {
  const clientName = document.getElementById('clientName').value.trim();

  // Build WebSocket URL - connect to same host
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  let url = `${protocol}//${window.location.host}/api/connect`;

  if (clientName) {
    url += `?client_name=${encodeURIComponent(clientName)}`;
  }

  ws = new WebSocket(url);

  ws.onopen = () => {
    setConnected(true);
    addMessage('system', 'Connected to server');
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      handleServerMessage(data);
    } catch (e) {
      addMessage('error', `Failed to parse message: ${event.data}`);
    }
  };

  ws.onclose = () => {
    setConnected(false);
    addMessage('system', 'Disconnected from server');
    subscriptions.clear();
    updateSubscriptionsUI();
  };

  ws.onerror = (error) => {
    addMessage('error', 'WebSocket error occurred');
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
  publishBtn.disabled = !connected || subscriptions.size === 0;
  document.getElementById('clientName').disabled = connected;
}

function handleServerMessage(data) {
  // Check if it's a chat message (has sender, channel_name, content, sent_at)
  if (data.sender && data.channel_name && data.content && data.sent_at) {
    const time = new Date(data.sent_at).toLocaleTimeString();
    addMessage('incoming', `[${time}] #${data.channel_name} - ${data.sender}: ${data.content}`);
    return;
  }

  // Otherwise it's a response/status message
  if (data.status === 'ok') {
    if (data.info) {
      if (data.info.client_name) {
        addMessage('system', `Connected as: ${data.info.client_name}`);
      }
      if (data.info.channel_name && data.info.total_subscribers !== undefined) {
        addMessage('system', `Subscribed to #${data.info.channel_name} (${data.info.total_subscribers} subscribers)`);
      }
    } else {
      addMessage('system', 'OK');
    }
  } else if (data.status === 'error') {
    const detail = data.info?.detail || 'Unknown error';
    addMessage('error', `Error: ${detail}`);
  } else {
    // Unknown message format, just display it
    addMessage('incoming', JSON.stringify(data, null, 2));
  }
}

function subscribe() {
  const channel = document.getElementById('subscribeChannel').value.trim();
  if (!channel) {
    addMessage('error', 'Please enter a channel name');
    return;
  }

  const msg = {
    action: 'subscribe',
    params: {
      channel_name: channel
    }
  };

  sendMessage(msg);
  subscriptions.add(channel);
  updateSubscriptionsUI();
  document.getElementById('subscribeChannel').value = '';
}

function unsubscribe(channel) {
  const msg = {
    action: 'unsubscribe',
    params: {
      channel_name: channel
    }
  };

  sendMessage(msg);
  subscriptions.delete(channel);
  updateSubscriptionsUI();
}

function publish() {
  const channel = publishChannelEl.value;
  const content = document.getElementById('messageContent').value.trim();

  if (!channel) {
    addMessage('error', 'Please select a channel');
    return;
  }
  if (!content) {
    addMessage('error', 'Please enter a message');
    return;
  }

  const msg = {
    action: 'publish',
    params: {
      channel_name: channel,
      content: content
    }
  };

  sendMessage(msg);
  addMessage('outgoing', `[Sent to #${channel}]: ${content}`);
  document.getElementById('messageContent').value = '';
}

function sendMessage(msg) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(msg));
  } else {
    addMessage('error', 'Not connected to server');
  }
}

function updateSubscriptionsUI() {
  if (subscriptions.size === 0) {
    subscriptionsEl.innerHTML = '<em style="color: #999;">None</em>';
    publishChannelEl.innerHTML = '<option value="">-- Subscribe first --</option>';
    publishChannelEl.disabled = true;
    publishBtn.disabled = true;
  } else {
    subscriptionsEl.innerHTML = '';
    publishChannelEl.innerHTML = '';

    subscriptions.forEach(channel => {
      // Add tag to subscriptions display
      const tag = document.createElement('span');
      tag.className = 'subscription-tag';
      tag.innerHTML = `#${channel} <span class="remove" onclick="unsubscribe('${channel}')">&times;</span>`;
      subscriptionsEl.appendChild(tag);

      // Add option to publish dropdown
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

function addMessage(type, text) {
  const div = document.createElement('div');
  div.className = `msg ${type}`;

  const now = new Date().toLocaleTimeString();
  div.innerHTML = `<div class="meta">${now}</div>${escapeHtml(text)}`;

  messagesEl.appendChild(div);
  messagesEl.scrollTop = messagesEl.scrollHeight;
}

function clearMessages() {
  messagesEl.innerHTML = '';
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Allow sending message with Enter key
document.getElementById('messageContent').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    publish();
  }
});

document.getElementById('subscribeChannel').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    subscribe();
  }
});

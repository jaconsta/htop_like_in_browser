import { h, render } from 'https://esm.sh/preact?module';
import htm from 'https://esm.sh/htm?module';

const html = htm.bind(h);

const App = (props) => {
  return html`<div>
    ${props.cpus.map(
    (cpu) =>
      html`<div class="bar">
          <div class="bar-inner" style="width: ${cpu}%"></div>
          <label>${cpu.toFixed(2)}%</label>
        </div>`
  )}
  </div>`;
};

const url = new URL('/ws/cpus/json', window.location.href);
url.protocol = url.protocol.replace('http', 'ws');
let ws = new WebSocket(url.href);
ws.onmessage = (ev) => {
  let json = JSON.parse(ev.data);
  render(html`<${App} cpus=${json}></${App}>`, document.body);
};

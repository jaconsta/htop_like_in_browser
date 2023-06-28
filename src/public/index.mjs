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

setInterval(async () => {
  let response = await fetch('/api/cpus/json');
  if (response.status !== 200) {
    const app = h('pre', null, 'error');
    render(app, document.body);
  }

  let json = await response.json();
  // const app = h('pre', null, JSON.stringify(json, null, 2));
  // render(app, document.body);
  render(html`<${App} cpus=${json}></${App}>`, document.body);
}, 1000);

document.addEventListener('DOMContentLoaded', () => {
  setInterval(async () => {
    let response = await fetch('/api/cpus/json');
    let json = await response.json();
    document.body.textContent = JSON.stringify(json, null, 2);
  }, 1000);
});

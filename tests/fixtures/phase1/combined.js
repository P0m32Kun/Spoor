// Phase 1 acceptance: fetch + location + XMLHttpRequest in one file.
fetch("/api/v1/users");
fetch("https://api.example.com/data", { method: "POST" });

location.href = "https://cdn.example.com/app.js";
location.replace("/login");
window.location = "/dashboard";

const xhr = new XMLHttpRequest();
xhr.open("GET", "/api/v1/status");
xhr.open("POST", "https://example.com/submit");

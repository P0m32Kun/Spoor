/**
 * Synthetic Katana extract — modern SPA chunk (React/Vite-like).
 * Covers: fetch, axios, router, graphql hint, dynamic URL noise.
 */
(function (global) {
  const API_BASE = "/api/v2";
  const token = "AKIAIOSFODNN7EXAMPLE";

  fetch("/api/v2/users?id=1&role=admin", { method: "GET" });
  fetch(API_BASE + "/orders");

  axios.post("https://billing.example.com/invoices", { amount: 100 });
  axios.get("/api/v2/profile");

  const routes = [
    { path: "/app/dashboard", element: null, loader: true },
    { path: "/app/settings/:tab", component: null },
  ];

  const query = gql`
    query Session {
      viewer { id email }
    }
  `;

  global.__SPOOR_SYNTHETIC_SPA__ = { token, routes, query };
})(typeof window !== "undefined" ? window : globalThis);

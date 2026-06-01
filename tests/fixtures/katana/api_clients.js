/**
 * Synthetic Katana extract — mixed HTTP clients + WebSocket.
 */
import ky from "ky";
import got from "got";

ky.get("https://realtime.example.com/api/v1/ping");
ky("/api/v1/health");

got.post("/api/v1/events");
got("https://hooks.example.com/callback", { method: "DELETE" });

superagent.get("/api/v1/agents");
request.put("/api/v1/agents/self");

const socket = new WebSocket("wss://realtime.example.com/ws");
new WebSocket("/api/v1/ws");

graphql("https://api.example.com/graphql", {});

/**
 * Synthetic Katana extract — negative controls.
 * Spoor should NOT emit EXPR endpoints or xhr from unrelated .open().
 */
const base = "/api/base";
fetch(base + "/dynamic/users");
fetch(base + "/dynamic/orders");

const db = { open: function (m, u) {} };
db.open("GET", "/not-an-xhr-endpoint");

const misc = {
  note: "AKIASHORT",
  url: "javascript:void(0)",
  data: "data:text/plain,hello",
};

const sourceMapRef = "//# sourceMappingURL=synthetic.bundle.js.map";

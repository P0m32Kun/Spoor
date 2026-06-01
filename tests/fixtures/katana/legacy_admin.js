/**
 * Synthetic Katana extract — legacy jQuery admin panel.
 */
var admin = admin || {};
admin.init = function () {
  $.get("/legacy/users/list");
  $.ajax({ url: "/legacy/users/export", type: "POST" });

  var xhr = new XMLHttpRequest();
  xhr.open("GET", "/legacy/session/check");
  xhr.open("POST", "https://auth.example.com/legacy/login");

  location.href = "/legacy/logout";
  location.replace("/legacy/home");
  window.open("/legacy/help-popup");
};

const vueRoutes = [
  { path: "/home", name: "home", component: Home },
  { path: "/users/:id", component: UserList },
];

createBrowserRouter([
  {
    path: "/admin",
    element: Admin,
    children: [{ path: "settings", element: Settings }],
  },
]);

const reactRoutes = [{ path: "/dashboard", element: Dashboard }];

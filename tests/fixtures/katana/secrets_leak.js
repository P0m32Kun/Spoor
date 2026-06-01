/**
 * Synthetic Katana extract — config / credential leaks (no real secrets).
 */
const standaloneGcp = "AIzaSy000000000000000000000000000000000";

const firebaseConfig = {
  apiKey: "AIzaSy1111111111111111111111111111111",
  authDomain: "demo.firebaseapp.com",
  projectId: "demo-project",
};

const serviceAccount = {
  type: "service_account",
  private_key:
    "-----BEGIN PRIVATE KEY-----\nLINE\n-----END PRIVATE KEY-----\n",
};

const appCfg = {
  apiKey: "sk-live-not-a-real-key-abcdef",
  password: "hardcoded-demo-password",
  token: "not-a-github-token",
};

const githubPat = "ghp_demoTokenNotReal123456789012345678";

export { standaloneGcp, firebaseConfig, serviceAccount, appCfg, githubPat };

using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "main", worker = .calfWorker),
    (name = "assets-service", disk = (path = "./public", writable = false))
  ],
  sockets = [
    (
      name = "http",
      address = "*:8080",
      http = (),
      service = "main"
    )
  ]
);

const calfWorker :Workerd.Worker = (
  compatibilityDate = "2026-04-05",
  modules = [
    (name = "index.js", esModule = embed "build/index.js"),
    (name = "index_bg.wasm", wasm = embed "build/index_bg.wasm")
  ],
  bindings = [
    (name = "AUTH_KEY", text = "your-secret-key"),
    (name = "ASSETS", service = "assets-service")
  ]
);

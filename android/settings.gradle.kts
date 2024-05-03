include(
    ":app",
    ":service",
    ":tile"
)
include(
    ":lib:account",
    ":lib:common",
    ":lib:endpoint",
    ":lib:intent-provider",
    ":lib:ipc",
    ":lib:model",
    ":lib:resource",
    ":lib:talpid",
    ":lib:theme",
    ":lib:common-test",
    ":lib:billing",
    ":lib:payment",
    ":lib:map",
    ":lib:daemon-grpc",
    ":lib:vpn-permission"
)
include(
    ":test",
    ":test:arch",
    ":test:common",
    ":test:e2e",
    ":test:mockapi"
)

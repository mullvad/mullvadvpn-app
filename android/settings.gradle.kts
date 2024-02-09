include(":app", ":service", ":tile")

include(
    ":lib:common",
    ":lib:endpoint",
    ":lib:ipc",
    ":lib:model",
    ":lib:resource",
    ":lib:talpid",
    ":lib:theme",
    ":lib:common-test",
    ":lib:billing",
    ":lib:payment",
    ":lib:map"
)

include(":test", ":test:arch", ":test:common", ":test:e2e", ":test:mockapi")

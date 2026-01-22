{ lib }:
{
  path = [
    "/run/wrappers"
    "/run/current-system/sw"
  ];
  serviceConfig = {
    DynamicUser = lib.mkForce [ ];
    SystemCallFilter = lib.mkForce [ ];
    RestrictNamespaces = lib.mkForce [ ];

    LockPersonality = lib.mkForce [ ];
    MemoryDenyWriteExecute = lib.mkForce [ ];
    NoNewPrivileges = lib.mkForce [ ];
    PrivateDevices = lib.mkForce [ ];
    PrivateMounts = lib.mkForce [ ];
    PrivateNetwork = lib.mkForce [ true ];
    PrivateTmp = lib.mkForce [ ];
    PrivateUsers = lib.mkForce [ ];
    ProcSubset = lib.mkForce [ ];
    ProtectClock = lib.mkForce [ ];
    ProtectControlGroups = lib.mkForce [ ];
    ProtectHome = lib.mkForce [ ];
    ProtectHostname = lib.mkForce [ ];
    ProtectKernelLogs = lib.mkForce [ ];
    ProtectKernelModules = lib.mkForce [ ];
    ProtectKernelTunables = lib.mkForce [ ];
    ProtectProc = "off";
    ProtectSystem = lib.mkForce [ ];
    RemoveIPC = lib.mkForce [ ];
    # Restart = lib.mkForce [];
    RestrictAddressFamilies = lib.mkForce [ ];
    RestrictRealtime = lib.mkForce [ ];
    RestrictSUIDSGID = lib.mkForce [ ];

    # If these are not set we will hit newuidmap/newgidmap permission issues when calling from
    # systemd service, we can most likely scope these permissions down way more.
    CapabilityBoundingSet = lib.mkForce [ "CAP_SYS_ADMIN CAP_SETUID CAP_SETGID CAP_DAC_OVERRIDE" ];
    AmbientCapabilities = lib.mkForce [ "CAP_SYS_ADMIN CAP_SETUID CAP_SETGID CAP_DAC_OVERRIDE" ];
  };
}

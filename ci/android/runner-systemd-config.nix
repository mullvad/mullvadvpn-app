{ lib }:
{
  path = [
    "/run/wrappers"
    "/run/current-system/sw"
  ];
  serviceConfig = {
    DynamicUser = lib.mkForce false;
    SystemCallFilter = lib.mkForce [ ];
    RestrictNamespaces = lib.mkForce false;
    LockPersonality = lib.mkForce false;
    MemoryDenyWriteExecute = lib.mkForce false;
    NoNewPrivileges = lib.mkForce false;
    PrivateDevices = lib.mkForce false;
    PrivateMounts = lib.mkForce false;
    PrivateNetwork = lib.mkForce false;
    PrivateTmp = lib.mkForce false;
    PrivateUsers = lib.mkForce false;
    ProcSubset = lib.mkForce "";
    ProtectClock = lib.mkForce false;
    ProtectControlGroups = lib.mkForce false;
    ProtectHome = lib.mkForce false;
    ProtectHostname = lib.mkForce false;
    ProtectKernelLogs = lib.mkForce false;
    ProtectKernelModules = lib.mkForce false;
    ProtectKernelTunables = lib.mkForce false;
    ProtectProc = "off";
    ProtectSystem = lib.mkForce false;
    RemoveIPC = lib.mkForce false;
    RestrictAddressFamilies = lib.mkForce [ ];
    RestrictRealtime = lib.mkForce false;
    RestrictSUIDSGID = lib.mkForce false;
    CapabilityBoundingSet = lib.mkForce [ "~CAP_SETGID" ];
    AmbientCapabilities = lib.mkForce [ "~CAP_SETGID" ];
  };
}

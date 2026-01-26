# Common systemd configuration options that are applied to all runner-0*
# systemd services that need to start run a job in the container with podman.
{ lib }:
{
  path = [
    "/run/wrappers"
    "/run/current-system/sw"
  ];
  serviceConfig = {
    # By default, the service config that we get from the nix github runner configuration
    # is too locked down for us to run start the container, so we have to unset some options
    # with lib.mkForce [].
    # We may be able to remove some of these lines.
    DynamicUser = lib.mkForce [ ];
    SystemCallFilter = lib.mkForce [ ];
    RestrictNamespaces = lib.mkForce [ ];
    LockPersonality = lib.mkForce [ ];
    MemoryDenyWriteExecute = lib.mkForce [ ];
    NoNewPrivileges = lib.mkForce [ ];
    PrivateDevices = lib.mkForce [ ];
    PrivateMounts = lib.mkForce [ ];
    PrivateNetwork = lib.mkForce [ ];
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
    RestrictAddressFamilies = lib.mkForce [ ];
    RestrictRealtime = lib.mkForce [ ];
    RestrictSUIDSGID = lib.mkForce [ ];

    # If these are not set we will hit newuidmap/newgidmap permission issues when calling from
    # systemd service.
    CapabilityBoundingSet = lib.mkForce [ "CAP_SYS_ADMIN CAP_SETUID CAP_SETGID CAP_DAC_OVERRIDE" ];
    AmbientCapabilities = lib.mkForce [ "CAP_SYS_ADMIN CAP_SETUID CAP_SETGID CAP_DAC_OVERRIDE" ];
  };
}

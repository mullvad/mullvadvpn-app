# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    ./hardware-configuration.nix
  ];

  # Bootloader.
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # Networking
  networking.hostName = "bender";
  networking.networkmanager.enable = true;

  # Set your time zone.
  time.timeZone = "Europe/Stockholm";

  # Select internationalisation properties.
  i18n.defaultLocale = "en_US.UTF-8";

  i18n.extraLocaleSettings = {
    LC_ADDRESS = "sv_SE.UTF-8";
    LC_IDENTIFICATION = "sv_SE.UTF-8";
    LC_MEASUREMENT = "sv_SE.UTF-8";
    LC_MONETARY = "sv_SE.UTF-8";
    LC_NAME = "sv_SE.UTF-8";
    LC_NUMERIC = "sv_SE.UTF-8";
    LC_PAPER = "sv_SE.UTF-8";
    LC_TELEPHONE = "sv_SE.UTF-8";
    LC_TIME = "sv_SE.UTF-8";
  };

  # Enable the X11 windowing system.
  services.xserver.enable = true;

  # Enable the GNOME Desktop Environment.
  services.displayManager.gdm.enable = true;
  services.desktopManager.gnome.enable = true;

  # Disable any hibernation
  systemd.targets.sleep.enable = false;
  systemd.targets.suspend.enable = false;
  systemd.targets.hibernate.enable = false;
  systemd.targets.hybrid-sleep.enable = false;

  # Configure keymap in X11
  services.xserver.xkb = {
    layout = "se";
    variant = "";
  };

  # Configure console keymap
  console.keyMap = "sv-latin1";

  users = {
    users = {
      runner-admin = {
        isNormalUser = true;
        extraGroups = [
          "networkmanager"
          "wheel"
        ];
        group = "runner-admin";
        packages = with pkgs; [
        ];
      };

      runner = {
        isNormalUser = true;
        description = "Runner user";
        extraGroups = [
          "podman"
          "networkmanager"
          "wheel"
          "runners"
          "docker"
        ];
        group = "runner";
        subUidRanges = [
          {
            startUid = 100000;
            count = 65536;
          }
        ];
        subGidRanges = [
          {
            startGid = 100000;
            count = 65536;
          }
        ];
        # needed for podman
        linger = true;
      };
    };

    groups = {
      runner-admin = { };
      runner = { };

      runners = {
        members = [ "runner" ];
      };
    };
  };

  # Install firefox.
  programs.firefox.enable = true;

  # Allow unfree packages
  nixpkgs.config.allowUnfree = true;

  # List packages installed in system profile. To search, run:
  # $ nix search wget
  environment.systemPackages = with pkgs; [
    neovim
    ncdu
    github-runner
    htop
    podman
    git
    javaPackages.compiler.openjdk17
    strace

    # Temp tests
    lm_sensors

    slirp4netns
  ];

  programs.neovim = {
    enable = true;
    defaultEditor = true;
    vimAlias = true;
    viAlias = true;
  };

  virtualisation = {
    containers.enable = true;
    containers.containersConf.settings = {
      containers.seccomp_profile = "/tmp/seccomp.json";
    };

    podman = {
      dockerSocket.enable = true;
      enable = true;
      dockerCompat = true;
      defaultNetwork.settings.dns_enabled = true;
      defaultNetwork.settings.driver = "slirp4netns";
    };
  };

  # Enable the OpenSSH daemon.
  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = false;
  };

  security.unprivilegedUsernsClone = true;
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    publish = {
      enable = true;
      addresses = true;
    };
  };

  services.github-runners = {
    android-bender-01 = {
      enable = true;
      name = "android-bender-01";
      tokenFile = "/home/runner/secrets/pat_token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner/android-bender-01";
    };
    # android-bender-02 = {
    #   enable = true;
    #   name = "android-bender-02";
    #   tokenFile = "/home/runner/secrets/pat_token";
    #   url = "https://github.com/mullvad/mullvadvpn-app";
    #   user = "runner";
    #   group = "runner";
    #   extraPackages = with pkgs; [
    #     podman
    #   ];
    #   workDir = "/home/runner/android-bender-02";
    # };
  };

  systemd.services."github-runner-android-bender-01" = {
    # Include system wrappers to expose 'newuidmap' and 'newgidmap'
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
      # Restart = lib.mkForce [];
      RestrictAddressFamilies = lib.mkForce [ ];
      RestrictRealtime = lib.mkForce [ ];
      RestrictSUIDSGID = lib.mkForce [ ];

      # If these are not set we will hit newuidmap/newgidmap permission issues when calling from
      # systemd service, we can most likely scope these permissions down way more.
      CapabilityBoundingSet = lib.mkForce [ "CAP_SETUID CAP_SETGID" ];
      AmbientCapabilities = lib.mkForce [ "CAP_SETUID CAP_SETGID" ];
    };
  };

  # This value determines the NixOS release from which the default
  # settings for stateful data, like file locations and database versions
  # on your system were taken. It‘s perfectly fine and recommended to leave
  # this value at the release version of the first install of this system.
  # Before changing this value read the documentation for this option
  # (e.g. man configuration.nix or on https://nixos.org/nixos/options.html).
  system.stateVersion = "25.11"; # Did you read the comment?
}

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
    # Include the results of the hardware scan.
    ./hardware-configuration.nix
  ];

  # Bootloader.
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking.hostName = "android-github-runner"; # Define your hostname.
  # networking.wireless.enable = true;  # Enables wireless support via wpa_supplicant.

  # Configure network proxy if necessary
  # networking.proxy.default = "http://user:password@proxy:port/";
  # networking.proxy.noProxy = "127.0.0.1,localhost,internal.domain";

  # Enable networking
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

  # Enable CUPS to print documents.
  services.printing.enable = true;

  # Enable sound with pipewire.
  services.pulseaudio.enable = false;
  security.rtkit.enable = true;

  # # These 2 lines might help us with the newuidmap permission error:
  # security.wrappers.newuidmap = {
  #   setuid = lib.mkForce false;
  #   capabilities = "cap_setuid+ep";
  # };
  # security.wrappers.newgidmap = {
  #   setuid = lib.mkForce false;
  #   capabilities = "cap_setgid+ep";
  # };

  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    # If you want to use JACK applications, uncomment this
    #jack.enable = true;

    # use the example session manager (no others are packaged yet so this is enabled by default,
    # no need to redefine it in your config for now)
    #media-session.enable = true;
  };

  # Enable touchpad support (enabled default in most desktopManager).
  # services.xserver.libinput.enable = true;

  # Define a user account. Don't forget to set a password with ‘passwd’.
  users.users.runner-admin = {
    isNormalUser = true;
    description = "Mole Runnersson";
    extraGroups = [
      "networkmanager"
      "wheel"
    ];
    group = "runner-admin";
    packages = with pkgs; [
      #  thunderbird
    ];
  };

  users.users.runner1 = {
    isNormalUser = true;
    description = "Runner1 user";
    extraGroups = [
      "podman"
      "networkmanager"
      "wheel"
      "runners"
      "docker"
    ];
    group = "runner1";
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

  users.groups.runner-admin = { };
  users.groups.runner1 = { };

  users.groups.runners = {
    members = [ "runner1" ];
  };

  # Install firefox.
  programs.firefox.enable = true;
  # programs.shadow.setuid = false;

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
    #  wget
    # Temp tests
    stress
    lm_sensors
    slirp4netns
  ];

  programs.java.enable = true;
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
    # docker.rootless.daemon.settings = {
    #   selinux-enabled = true;
    #   seccomp-profile = "unconfined";
    # };
    podman = {
      dockerSocket.enable = true;
      enable = true;
      dockerCompat = true;
      defaultNetwork.settings.dns_enabled = true;
      defaultNetwork.settings.driver = "slirp4netns";
    };
  };
  # virtualisation.oci-containers.backend = "podman"

  # Some programs need SUID wrappers, can be configured further or are
  # started in user sessions.
  # programs.mtr.enable = true;
  # programs.gnupg.agent = {
  #   enable = true;
  #   enableSSHSupport = true;
  # };

  # List services that you want to enable:

  # Enable the OpenSSH daemon.
  services.openssh.enable = true;

  security.unprivilegedUsernsClone = true;
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    publish = {
      enable = true;
      addresses = true;
    };
  };

  systemd.services."github-runner-android-runner-1" = {
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
      CapabilityBoundingSet = lib.mkForce [ "~" ];
      AmbientCapabilities = lib.mkForce [ "~" ];
    };
  };

  services.github-runners = {
    android-runner-1 = {
      enable = true;
      # To be changed later, for final setup
      name = "android-runner-1-test";
      tokenFile = "/etc/nixos/secrets/android-runner-1-token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner1";
      group = "runner1";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner1/runner1";
    };
  };

  # Open ports in the firewall.
  # networking.firewall.allowedTCPPorts = [ ... ];
  # networking.firewall.allowedUDPPorts = [ ... ];
  # Or disable the firewall altogether.
  # networking.firewall.enable = false;

  # This value determines the NixOS release from which the default
  # settings for stateful data, like file locations and database versions
  # on your system were taken. It‘s perfectly fine and recommended to leave
  # this value at the release version of the first install of this system.
  # Before changing this value read the documentation for this option
  # (e.g. man configuration.nix or on https://nixos.org/nixos/options.html).
  system.stateVersion = "25.11"; # Did you read the comment?
}

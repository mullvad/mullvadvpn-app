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
      tokenFile = "/home/runner/.registration-tokens/android-bender-01.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner/android-bender-01";
      extraLabels = [
        "android-bender-01"
        "android-build"
      ];
    };
    android-bender-02 = {
      enable = true;
      name = "android-bender-02";
      tokenFile = "/home/runner/.registration-tokens/android-bender-02.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner/android-bender-02";
      extraLabels = [
        "android-bender-02"
        "android-build"
      ];
    };
    android-bender-03 = {
      enable = true;
      name = "android-bender-03";
      tokenFile = "/home/runner/.registration-tokens/android-bender-03.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner/android-bender-03";
      extraEnvironment = {
        ANDROID_SERIAL = "29121FDH200A6G";
      };
      extraLabels = [
        "android-bender-03"
        "android-device-test"
      ];
    };
  };

  systemd.services."github-runner-android-bender-01" = import ./runner-systemd-config.nix {
    inherit lib;
  };
  systemd.services."github-runner-android-bender-02" = import ./runner-systemd-config.nix {
    inherit lib;
  };
  systemd.services."github-runner-android-bender-03" = import ./runner-systemd-config.nix {
    inherit lib;
  };

  # This value determines the NixOS release from which the default
  # settings for stateful data, like file locations and database versions
  # on your system were taken. It‘s perfectly fine and recommended to leave
  # this value at the release version of the first install of this system.
  # Before changing this value read the documentation for this option
  # (e.g. man configuration.nix or on https://nixos.org/nixos/options.html).
  system.stateVersion = "25.11"; # Did you read the comment?
}

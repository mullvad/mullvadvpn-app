# Edit this configuration file to define what should be installed on

# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{
  pkgs,
  lib,
  ...
}:

{
  imports = [
    ./hardware-configuration.nix
  ];

  # Bootloader.
  boot.loader = {
    systemd-boot.enable = true;
    efi.canTouchEfiVariables = true;
  };

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

  services = {
    # Enable the X11 windowing system.
    xserver = {
      enable = true;
      # Configure keymap in X11
      xkb = {
        layout = "se";
        variant = "";
      };
    };

    # Enable the GNOME Desktop Environment.
    displayManager.gdm.enable = true;
    desktopManager.gnome.enable = true;

    udev = {
      # Setup udev rules for our E2E test runner devices. When a device is plugged in the corresponding
      # github runner service is started, and if the device is removed the service is stopped.
      # Each runner will only execute on a specific devices as specified with ATTR{serial}.
      # Note: the serial '43161JEKB02504' is temporary until we get the real runner-08 device.
      extraRules = ''
        SUBSYSTEM=="usb", ATTR{serial}=="29121FDH200A6G", ACTION=="add|bind", \
          TAG+="systemd", SYMLINK="android5", ENV{SYSTEMD_WANTS}+="github-runner-android-bender-05.service" \
          OWNER="runner05", MODE="0660"

        SUBSYSTEM=="usb", ATTR{serial}=="3B111JEHN07568", ACTION=="add|bind", \
          TAG+="systemd", SYMLINK="android6", ENV{SYSTEMD_WANTS}+="github-runner-android-bender-06.service" \
          OWNER="runner06", MODE="0660"

        SUBSYSTEM=="usb", ATTR{serial}=="43151JEKB14933", ACTION=="add|bind", \
          TAG+="systemd", SYMLINK="android7", ENV{SYSTEMD_WANTS}+="github-runner-android-bender-07.service" \
          OWNER="runner07", MODE="0660"

        SUBSYSTEM=="usb", ATTR{serial}=="43161JEKB02504", ACTION=="add|bind", \
          TAG+="systemd", SYMLINK="android8", ENV{SYSTEMD_WANTS}+="github-runner-android-bender-08.service" \
          OWNER="runner08", MODE="0660"
      '';
    };
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

      # We create one user per runner. The reason we do this is because
      # we ran into conflicts with aardvark-dns and pasta trying to create conflicting
      # resources such as port binds and /dev/net/tap access when running multiple container
      # instances in parallel using a single runner user.
      # There is probably a better way to solve this.
      runner01 = {
        isNormalUser = true;
        description = "Runner user 01";
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

      runner02 = {
        isNormalUser = true;
        description = "Runner user 02";
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

      runner03 = {
        isNormalUser = true;
        description = "Runner user 03";
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

      runner04 = {
        isNormalUser = true;
        description = "Runner user 04";
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

      runner05 = {
        isNormalUser = true;
        description = "Runner user 05";
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

      runner06 = {
        isNormalUser = true;
        description = "Runner user 06";
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

      runner07 = {
        isNormalUser = true;
        description = "Runner user 07";
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

      runner08 = {
        isNormalUser = true;
        description = "Runner user 08";
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
    strace
    findutils
    android-tools

    # Temp tests
    lm_sensors
  ];

  programs.neovim = {
    enable = true;
    defaultEditor = true;
    vimAlias = true;
    viAlias = true;
  };

  virtualisation = {
    containers = {
      enable = true;
      # We have an issue where the container is not able to stop when it gets
      # a SIGTERM so the runner waits for 10 secs, then sends a SIGKILL which stops
      # the container and the build completes.
      # We think setting the following option would solve having to wait 10 extra seconds,
      # but this setting would be applied to the runner-admin podman config (/etc/containers/containers.conf),
      # and now the runner-0* users' podman configs. We should figure out how to set this
      # correctly in the user configs. We may have to use Nix Home Manager to set this up.
      #containersConf.settings = {
      #  StopSignal = 9;
      #};

      containersConf.settings = {
        containers = {
          # Increase pids_limit, otherwise ktfmt would always fail
          pids_limit = 8096;
        };
      };
    };
    podman = {
      dockerSocket.enable = true;
      enable = true;
      dockerCompat = true;
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
      tokenFile = "/home/runner01/.registration-token/android-bender-01.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner01";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner01/android-bender-01";
      extraLabels = [
        "android-bender-01"
        "android-build"
      ];
    };

    android-bender-02 = {
      enable = true;
      name = "android-bender-02";
      tokenFile = "/home/runner02/.registration-token/android-bender-02.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner02";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner02/android-bender-02";
      extraLabels = [
        "android-bender-02"
        "android-build"
      ];
    };

    android-bender-03 = {
      enable = true;
      name = "android-bender-03";
      tokenFile = "/home/runner03/.registration-token/android-bender-03.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner03";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner03/android-bender-03";
      extraLabels = [
        "android-bender-03"
        "android-build"
      ];
    };

    android-bender-04 = {
      enable = true;
      name = "android-bender-04";
      tokenFile = "/home/runner04/.registration-token/android-bender-04.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner04";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner04/android-bender-04";
      extraLabels = [
        "android-bender-04"
        "android-build"
      ];
    };

    android-bender-05 = {
      enable = true;
      name = "android-bender-05";
      tokenFile = "/home/runner05/.registration-token/android-bender-05.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner05";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner05/android-bender-05";
      extraEnvironment = {
        ANDROID_SERIAL = "29121FDH200A6G";
      };
      extraLabels = [
        "android-bender-05"
        "android-device"
      ];
    };

    android-bender-06 = {
      enable = true;
      name = "android-bender-06";
      tokenFile = "/home/runner06/.registration-token/android-bender-06.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner06";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner06/android-bender-06";
      extraEnvironment = {
        ANDROID_SERIAL = "3B111JEHN07568";
      };
      extraLabels = [
        "android-bender-06"
        "android-device"
      ];
    };

    android-bender-07 = {
      enable = true;
      name = "android-bender-07";
      tokenFile = "/home/runner07/.registration-token/android-bender-07.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner07";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner07/android-bender-07";
      extraEnvironment = {
        ANDROID_SERIAL = "43151JEKB14933";
      };
      extraLabels = [
        "android-bender-07"
        "android-device"
      ];
    };

    android-bender-08 = {
      enable = true;
      name = "android-bender-08";
      tokenFile = "/home/runner08/.registration-token/android-bender-08.token";
      url = "https://github.com/mullvad/mullvadvpn-app";
      user = "runner08";
      group = "runner";
      extraPackages = with pkgs; [
        podman
      ];
      workDir = "/home/runner08/android-bender-08";
      extraEnvironment = {
        ANDROID_SERIAL = "43161JEKB02504";
      };
      extraLabels = [
        "android-bender-08"
        "android-device"
      ];
    };
  };

  systemd = {
    targets = {
      # Disable any hibernation
      sleep.enable = false;
      suspend.enable = false;
      hibernate.enable = false;
      hybrid-sleep.enable = false;
    };

    services = {
      "github-runner-android-bender-01" = import ./runner-systemd-config.nix {
        inherit lib;
      };
      "github-runner-android-bender-02" = import ./runner-systemd-config.nix {
        inherit lib;
      };
      "github-runner-android-bender-03" = import ./runner-systemd-config.nix {
        inherit lib;
      };
      "github-runner-android-bender-04" = import ./runner-systemd-config.nix {
        inherit lib;
      };
      "github-runner-android-bender-05" = (import ./runner-systemd-config.nix { inherit lib; }) // {
        wantedBy = lib.mkForce [ ];
        bindsTo = [ "dev-android5.device" ];
        after = [ "dev-android5.device" ];
      };
      "github-runner-android-bender-06" = (import ./runner-systemd-config.nix { inherit lib; }) // {
        wantedBy = lib.mkForce [ ];
        bindsTo = [ "dev-android6.device" ];
        after = [ "dev-android6.device" ];
      };
      "github-runner-android-bender-07" = (import ./runner-systemd-config.nix { inherit lib; }) // {
        wantedBy = lib.mkForce [ ];
        bindsTo = [ "dev-android7.device" ];
        after = [ "dev-android7.device" ];
      };
      "github-runner-android-bender-08" = (import ./runner-systemd-config.nix { inherit lib; }) // {
        wantedBy = lib.mkForce [ ];
        bindsTo = [ "dev-android8.device" ];
        after = [ "dev-android8.device" ];
      };
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

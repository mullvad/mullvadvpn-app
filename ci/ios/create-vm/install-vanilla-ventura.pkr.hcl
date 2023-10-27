packer {
  required_plugins {
    tart = {
      version = ">= 1.2.0"
      source  = "github.com/cirruslabs/tart"
    }
  }
}

variable "vm_name" { type = string }

source "tart-cli" "tart" {
  # You can find macOS IPSW URLs on various websites like https://ipsw.me/
  # and https://www.theiphonewiki.com/wiki/Beta_Firmware/Mac/13.x
  from_ipsw    = "https://updates.cdn-apple.com/2023SummerFCS/fullrestores/042-43686/945D434B-DA5D-48DB-A558-F6D18D11AD69/UniversalMac_13.5.2_22G91_Restore.ipsw"
  vm_name      = "${var.vm_name}"
  cpu_count    = 4
  memory_gb    = 8
  disk_size_gb = 60
  ssh_password = "admin"
  ssh_username = "admin"
  ssh_timeout  = "120s"
   boot_command = [
    # hello, hola, bonjour, etc.
    "<wait60s><spacebar>",
    # Force selection of english as a language.
    "<wait30s>english<enter>",
    # Select Your Country and Region
    "<wait30s>united states<leftShiftOn><tab><leftShiftOff><spacebar>",
    # Written and Spoken Languages
    "<wait10s><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Accessibility
    "<wait10s><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Data & Privacy
    "<wait10s><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Migration Assistant
    "<wait10s><tab><tab><tab><spacebar>",
    # Sign In with Your Apple ID
    "<wait10s><leftShiftOn><tab><leftShiftOff><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Are you sure you want to skip signing in with an Apple ID?
    "<wait10s><tab><spacebar>",
    # Terms and Conditions
    "<wait10s><leftShiftOn><tab><leftShiftOff><spacebar>",
    # I have read and agree to the macOS Software License Agreement
    "<wait10s><tab><spacebar>",
    # Create a Computer Account
    "<wait10s>admin<tab><tab>admin<tab>admin<tab><tab><tab><spacebar>",
    # Enable Location Services
    "<wait10s><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Are you sure you don't want to use Location Services?
    "<wait10s><tab><spacebar>",
    # Select Your Time Zone
    "<wait10s><tab>UTC<enter><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Analytics
    "<wait10s><tab><spacebar><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Screen Time
    "<wait10s><tab><spacebar>",
    # Siri
    "<wait10s><tab><spacebar><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Choose Your Look
    "<wait10s><leftShiftOn><tab><leftShiftOff><spacebar>",
    # Enable Voice Over
    "<wait10s><leftAltOn><f5><leftAltOff><wait5s>v",
    # Now that the installation is done, open "System Settings"
    "<wait10s><leftAltOn><spacebar><leftAltOff>System Settings<enter>",
    # Focus the Search pane
    "<wait10s><leftAltOn>f<leftAltOff>",
    # Navigate to "Sharing"
    "<wait10s>sharing<enter><wait5s><down>",
    # Enable Screen Sharing
    "<wait10s><tab><spacebar>",
    # Enable Remote Login
    "<wait10s><tab><tab><tab><tab><tab><tab><spacebar>",
    # Disable Voice Over
    "<leftAltOn><f5><leftAltOff>",
  ]

  // A (hopefully) temporary workaround for Virtualization.Framework's
  // installation process not fully finishing in a timely manner
  create_grace_time = "30s"
}

build {
  sources = ["source.tart-cli.tart"]

  provisioner "shell" {
    inline = [
      // Enable passwordless sudo
      "echo admin | sudo -S sh -c \"mkdir -p /etc/sudoers.d/; echo 'admin ALL=(ALL) NOPASSWD: ALL' | EDITOR=tee visudo /etc/sudoers.d/admin-nopasswd\"",
      // Enable auto-login
      //
      // See https://github.com/xfreebird/kcpassword for details.
      "echo '00000000: 1ced 3f4a bcbc ba2c caca 4e82' | sudo xxd -r - /etc/kcpassword",
      "sudo defaults write /Library/Preferences/com.apple.loginwindow autoLoginUser admin",
      // Disable screensaver at login screen
      "sudo defaults write /Library/Preferences/com.apple.screensaver loginWindowIdleTime 0",
      // Disable screensaver for admin user
      "defaults -currentHost write com.apple.screensaver idleTime 0",
      // Prevent the VM from sleeping
      "sudo systemsetup -setdisplaysleep Off",
      "sudo systemsetup -setsleep Off",
      "sudo systemsetup -setcomputersleep Off",
      // Launch Safari to populate the defaults
      "/Applications/Safari.app/Contents/MacOS/Safari &",
      "sleep 30",
      "kill -9 %1",
      // Enable Safari's remote automation and "Develop" menu
      "sudo safaridriver --enable",
      "defaults write com.apple.Safari.SandboxBroker ShowDevelopMenu -bool true",
      "defaults write com.apple.Safari IncludeDevelopMenu -bool true",
      // Disable screen lock
      //
      // Note that this only works if the user is logged-in,
      // i.e. not on login screen.
      "sysadminctl -screenLock off -password admin",
      "defaults -currentHost write com.apple.screensaver idleTime 0"
    ]
  }
}

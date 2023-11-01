packer {
  required_plugins {
    tart = {
      version = ">= 1.2.0"
      source  = "github.com/cirruslabs/tart"
    }
  }
}

variable "shared_folder_path" { type = string }

variable "xcode_version" {
  type = string

  validation {
    condition = can(regex("(\\d)+(\\.)?((\\d)+)?(\\.)?((\\d)+)?", var.xcode_version))
    error_message = "Invalid Xcode version number. Example of a valid number: '15.0.1'."
  }
}

variable "vm_name" { type = string }

variable "user_name" { type = string }

variable "xcode_xip_name" {
  type = string

  validation {
    condition = can(regex("Xcode_(\\d)+(\\.)?((\\d)+)?(\\.)?((\\d)+)?\\.xip", var.xcode_xip_name))
    error_message = "Invalid Xcode file name. Example of a valid file name: 'Xcode_15.0.1.xip'."
  }
}

source "tart-cli" "tart" {
  vm_name      = "${var.vm_name}"
  ssh_password = "admin"
  ssh_username = "admin"
  ssh_timeout  = "120s"
  disk_size_gb = 80
}

build {
  sources = ["source.tart-cli.tart"]


  // Create a symlink for bash compatibility
  provisioner "shell" {
    script = "scripts/link-zprofile.sh"
  }

  // Install brew
  provisioner "shell" {
    environment_vars = [
    "USER=${var.user_name}"
    ]
    script = "scripts/install-brew.sh"
  }

  // Install required brew dependencies
  provisioner "shell" {
    script = "scripts/install-brew-dependencies.sh"
  }

  // Install rustup
  provisioner "shell" {
    script = "scripts/install-rustup.sh"
  }

  // Install go
  provisioner "shell" {
    script = "scripts/install-go.sh"
  }

  // Copy the local Xcode xip file to the VM
  provisioner "file" {
    source      = "${var.shared_folder_path}/${var.xcode_xip_name}"
    destination = "/tmp/${var.xcode_xip_name}"
  }

  // Install Xcode via xcodes.app
  provisioner "shell" {

    environment_vars = [
    "XCODE_VERSION=${var.xcode_version}",
    "XCODE_XIP_NAME=${var.xcode_xip_name}",
    "XCODE_SHARED_PATH=/tmp",
    ]
    script = "scripts/install-xcode.sh"
  }

  // Delete the Xcode xip file to save some space
  provisioner "shell" {
    inline = [
      "rm -f /tmp/${var.xcode_xip_name}"
    ]
  }

  // Run the xcodebuild first launch prompt to automatically accept terms and conditions, and download the iOS runtime simulator
  provisioner "shell" {
    script = "scripts/run-xcode-first-launch.sh"
  }

  // Remove everything in case of error
  error-cleanup-provisioner "shell" {
    script = "scripts/cleanup.sh"
  }
}

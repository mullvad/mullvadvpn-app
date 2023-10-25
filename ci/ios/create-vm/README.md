### Creating new macOS VMs to build MullvadVPN iOS in a CI environment
This guide assumes you are running on macOS.
## Prerequisites
In order to create VMs on the fly, we decided to use [tart](https://tart.run/) and [packer](https://developer.hashicorp.com/packer).

The various scripts that run in the VM are written in bash with the help of [shellcheck](shellcheck.net).

# VM requirements
- You will need at least 60GB of available space on your VM host
- You will need at least 8GB of available RAM on your VM host
- You will need at least 4 CPU cores available on your VM host

# How to install Tart
- brew install `cirruslabs/cli/tart`

# How to install Packer
- brew tap `hashicorp/tap`
- brew install `hashicorp/tap/packer`

# How to install shellcheck
- brew install `shellcheck`

> [!IMPORTANT] 
> # Prerequisite setup before running packer
- Get a copy of the Xcode version you want to install on the VM in a xip format
- Copy that file into the folder named `vm_shared_folder`
- Open the file named `variables.pkrvars.hcl`
- Edit the variables named `xcode_version` and `xcode_xip_name`

Here is an example of what to expect
```bash
% ls vm_shared_folder
Xcode_15.0.1.xip
% head -2 variables.pkrvars.hcl
xcode_version = "15.0.1"
xcode_xip_name = "Xcode_15.0.1.xip"
```

# Sanity checks before running packer
It is a good idea to keep logs, the `logs` folder is provided to that effect.
Enable packer logs by setting the following environment variables (assuming your are running with `zsh`)
- export `PACKER_LOG=1`
- export `PACKER_LOG_PATH="logs/packer_logs.txt"`

> [!NOTE] 
The logs will be overwritten with each packer command you issue.

You can then check that the templates are valid before running `packer` 
- packer inspect `-var-file="variables.pkrvars.hcl" install-xcode.pkr.hcl`
- packer validate `-var-file="variables.pkrvars.hcl" install-xcode.pkr.hcl`

# Create the VM image via Packer
Once your setup is ready, you just need one command to create a VM. And one more to install Xcode on it.
- packer build `-var-file="variables.pkrvars.hcl" install-vanilla-ventura.pkr.hcl`

# Install Xcode on the VM image via Packer
- packer build `-var-file="variables.pkrvars.hcl" install-xcode.pkr.hcl`

> [!IMPORTANT]
> At the time of writing this, `tart` does not support VM snapshotting. This means that any action taken by packer will be **permanent** on the VM.

Make sure to properly clean up the VM before running packer commands again if something went wrong.
You can look at the `cleanup.sh` script in the `scripts` folder to see what type of cleanup is ran in case things go wrong.
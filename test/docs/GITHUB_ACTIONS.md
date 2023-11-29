# `test-manager` in Github Actions

There exists a Workflow for running `test-manager` on a selection of different
platforms. The Workflow file can be found
[here](https://github.com/mullvad/mullvadvpn-app/blob/main/.github/workflows/desktop-e2e.yml).

## Adding a new platform

Basically everything related to adding a new virtual machine (VM) to the Workflow is a manual process, but it is fairly straightforward. Most steps (1-3) are performed outside of git, on both your local machine (1) as well as the GitHub runner (2-3). The last step (4) warrants a pull request, as it will make change(s) to the Workflow file.

1. Create a new VM based on [these instructions](./BUILD_OS_IMAGE.md)
2. Upload the newly-assembled VM to the GitHub Actions runner
3. Add an entry for the VM in the [test-manager config file](../test-manager/docs/config.md) on the GitHub runner
4. Update [this Workflow](https://github.com/mullvad/mullvadvpn-app/blob/main/.github/workflows/desktop-e2e.yml) in the [Mullvad App repository](https://github.com/mullvad/mullvadvpn-app/). This will enable GitHub Actions to trigger the `test-manager` with the new VM!

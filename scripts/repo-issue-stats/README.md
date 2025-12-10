# Github repository issue stats visualization

These scripts create a static html page where you can view a graph over the number of open github issues on this repository. This static html file is regenerated nightly.

When running `update_issues.py` the first time to bootstrap `mullvadvpn-app.issues/` you need to run it with a github token to be able to fetch all issues without getting rate limited. Subsequent updates (done by the systemd timer) can run without a token, since there usually are not that many new issues.

```
GITHUB_TOKEN=$(gh auth token) ./update_issues.py mullvad/mullvadvpn-app mullvadvpn-app.issues/
```

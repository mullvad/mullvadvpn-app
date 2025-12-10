# Installing systemd timer for updating the static html file

## Installation steps

1. **Copy the service and timer files to systemd directory:**
   ```bash
   sudo cp update-issues.service update-issues.timer /etc/systemd/system/
   ```

2. **Reload systemd to recognize new files:**
   ```bash
   sudo systemctl daemon-reload
   ```

3. **Enable and start the timer:**
   ```bash
   sudo systemctl enable update-issues.timer
   sudo systemctl start update-issues.timer
   ```

## Uninstall

```bash
sudo systemctl stop update-issues.timer
sudo systemctl disable update-issues.timer
sudo rm /etc/systemd/system/update-issues.service /etc/systemd/system/update-issues.timer
sudo rm sudo systemctl daemon-reload
```

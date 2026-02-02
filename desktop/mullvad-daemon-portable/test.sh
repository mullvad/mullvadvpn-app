systemctl   stop     mullvad-portable.service
portablectl detach ./mullvad-portable.service
portablectl attach --profile=trusted ./mullvad-portable.service
systemctl   start    mullvad-portable.service
systemctl   status   mullvad-portable.service
journalctl -xefu     mullvad-portable.service


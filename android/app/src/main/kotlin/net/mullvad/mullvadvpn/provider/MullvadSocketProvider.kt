package net.mullvad.mullvadvpn.provider

import androidx.core.content.FileProvider
import net.mullvad.mullvadvpn.R

class MullvadSocketProvider : FileProvider(R.xml.socket_paths)

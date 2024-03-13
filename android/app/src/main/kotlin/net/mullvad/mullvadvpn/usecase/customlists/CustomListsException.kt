package net.mullvad.mullvadvpn.usecase.customlists

import net.mullvad.mullvadvpn.model.CustomListsError

class CustomListsException(val error: CustomListsError) : Throwable()

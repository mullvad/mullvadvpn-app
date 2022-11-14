package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository

class PrivacyDisclaimerViewModel(
    private val privacyDisclaimerRepository: PrivacyDisclaimerRepository
) : ViewModel() {
    fun setPrivacyDisclosureAccepted() = privacyDisclaimerRepository.setPrivacyDisclosureAccepted()
}

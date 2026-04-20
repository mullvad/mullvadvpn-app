package net.mullvad.mullvadvpn.feature.applisting.impl

import android.content.res.Resources
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.lib.model.PackageName
import net.mullvad.mullvadvpn.lib.ui.resource.R
import org.junit.jupiter.api.Test

class ResolveAppListingUseCaseImplTest {

    private val mockResources: Resources = mockk()

    @Test
    fun `when play build should return market uri`() {
        // Arrange
        val useCase = createUseCase(isPlayBuild = true, isStoreInstall = false)

        // Act
        val result = useCase()

        // Assert
        assertEquals(MARKET_URI, result.listingUri)
        assertEquals(MARKET_ERROR, result.errorMessage)
    }

    @Test
    fun `when store install should return market uri`() {
        // Arrange
        val useCase = createUseCase(isPlayBuild = false, isStoreInstall = true)

        // Act
        val result = useCase()

        // Assert
        assertEquals(MARKET_URI, result.listingUri)
        assertEquals(MARKET_ERROR, result.errorMessage)
    }

    @Test
    fun `when sideloaded build should return download url`() {
        // Arrange
        val useCase = createUseCase(isPlayBuild = false, isStoreInstall = false)

        // Act
        val result = useCase()

        // Assert
        assertEquals(DOWNLOAD_URL, result.listingUri)
        assertEquals(BROWSER_ERROR, result.errorMessage)
    }

    private fun createUseCase(
        isPlayBuild: Boolean,
        isStoreInstall: Boolean,
    ): ResolveAppListingUseCaseImpl {
        every { mockResources.getString(R.string.market_uri, PACKAGE_NAME.value) } returns
            MARKET_URI
        every { mockResources.getString(R.string.download_url) } returns DOWNLOAD_URL
        every { mockResources.getString(R.string.uri_market_app_not_found) } returns MARKET_ERROR
        every { mockResources.getString(R.string.uri_browser_app_not_found) } returns BROWSER_ERROR

        return ResolveAppListingUseCaseImpl(
            resources = mockResources,
            packageName = PACKAGE_NAME,
            isPlayBuild = isPlayBuild,
            installSourceProvider = InstallSourceProvider { isStoreInstall },
        )
    }

    companion object {
        private val PACKAGE_NAME = PackageName("net.mullvad.mullvadvpn")
        private const val MARKET_URI = "market://details?id=net.mullvad.mullvadvpn"
        private const val DOWNLOAD_URL = "https://mullvad.net/download/vpn/android"
        private const val MARKET_ERROR = "No Android app store installed, could not open link"
        private const val BROWSER_ERROR = "No browser app installed, could not open link"
    }
}

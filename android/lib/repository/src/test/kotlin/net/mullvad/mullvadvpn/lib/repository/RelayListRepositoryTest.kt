package net.mullvad.mullvadvpn.lib.repository

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import java.util.Locale
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.RelayItem
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

@ExperimentalCoroutinesApi
class RelayListRepositoryTest {
    private val mockManagementService: ManagementService = mockk()
    private val mockTranslationRepository: RelayLocationTranslationRepository = mockk()

    private lateinit var relayListRepository: RelayListRepository
    private lateinit var defaultLocale: Locale

    private val countriesFlow = MutableStateFlow<List<RelayItem.Location.Country>>(emptyList())
    private val translationsFlow = MutableStateFlow<Translations>(emptyMap())

    @BeforeEach
    fun setup() {
        defaultLocale = Locale.getDefault()
        every { mockManagementService.relayCountries } returns countriesFlow
        // Unused by these tests, but read by other RelayListRepository properties on construction.
        every { mockManagementService.wireguardEndpointData } returns emptyFlow()
        every { mockManagementService.settings } returns emptyFlow()
        every { mockTranslationRepository.translations } returns translationsFlow
        relayListRepository =
            RelayListRepository(
                managementService = mockManagementService,
                translationRepository = mockTranslationRepository,
                dispatcher = UnconfinedTestDispatcher(),
            )
    }

    @AfterEach
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun `countries should be sorted using the locale's collation`() = runTest {
        // Arrange
        // 'Á' (U+00C1) outranks 'Z' (U+005A) by Unicode value, so comparing names by Unicode
        // value would place the accented countries after "Zimbabwe".
        Locale.setDefault(Locale.forLanguageTag("pt"))
        val countries = listOf(country("Zimbabwe"), country("Áustria"), country("Albânia"))
        countriesFlow.value = countries
        translationsFlow.value = countries.identityTranslations()

        // Act, Assert
        relayListRepository.relayList.test {
            assertEquals(listOf("Albânia", "Áustria", "Zimbabwe"), awaitItem().map { it.name })
        }
    }

    @Test
    fun `cities should be sorted using the locale's collation`() = runTest {
        // Arrange
        Locale.setDefault(Locale.forLanguageTag("pt"))
        val countries = listOf(country("Espanha", listOf("Zaragoza", "Ávila", "Barcelona")))
        countriesFlow.value = countries
        translationsFlow.value = countries.identityTranslations()

        // Act, Assert
        relayListRepository.relayList.test {
            assertEquals(
                listOf("Ávila", "Barcelona", "Zaragoza"),
                awaitItem().single().cities.map { it.name },
            )
        }
    }

    private fun country(name: String, cityNames: List<String> = emptyList()) =
        RelayItem.Location.Country(
            id = GeoLocationId.Country(name),
            name = name,
            cities = cityNames.map { city(it, countryName = name) },
        )

    private fun city(name: String, countryName: String) =
        RelayItem.Location.City(
            id = GeoLocationId.City(GeoLocationId.Country(countryName), name),
            name = name,
            latLong = LatLong(0f, 0f),
            relays = emptyList(),
            countryName = countryName,
        )

    // Relays are only sorted once their names have been translated, so map every name to itself to
    // keep the names unchanged while still exercising the translated-and-sorted path.
    private fun List<RelayItem.Location.Country>.identityTranslations(): Translations =
        flatMap { country ->
            listOf(country.name) + country.cities.map { it.name }
        }
        .associateWith { it }
}

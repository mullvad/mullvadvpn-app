package net.mullvad.mullvadvpn.feature.multihopmigration.impl

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.feature.multihopmigration.api.MultihopMigrationNavKey
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationData
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationState
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.PreviousDaitaState
import net.mullvad.mullvadvpn.lib.model.SplitFilterMigration
import net.mullvad.mullvadvpn.lib.repository.MultihopMigrationRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class MultihopMigrationViewModelTest {
    private val mockMultihopMigrationRepository = mockk<MultihopMigrationRepository>()
    private val mockWireguardConstraintsRepository = mockk<WireguardConstraintsRepository>()

    private lateinit var viewModel: MultihopMigrationViewModel

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `when calling nextPage should update currentPage with 1`() = runTest {
        // Arrange
        val multihopMigrationData =
            MultihopMigrationData(
                splitFilterMigration =
                    SplitFilterMigration(
                        multihopMigrationState = MultihopMigrationState.OFF_TO_WHEN_NEEDED,
                        daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                        filtersSet = true,
                    ),
                userBlocked = false,
            )
        viewModel = init(multihopMigrationData = multihopMigrationData)

        // Act, Assert
        viewModel.uiState.test {
            // Initial state
            val initialState = awaitItem()
            assert(initialState.currentPageIndex == 0)
            // After nextPage
            viewModel.nextPage()
            val nextState = awaitItem()
            assert(nextState.currentPageIndex == 1)
        }
    }

    @Test
    fun `when calling previousPage should update currentPage with -1`() = runTest {
        // Arrange
        val multihopMigrationData =
            MultihopMigrationData(
                splitFilterMigration =
                    SplitFilterMigration(
                        multihopMigrationState = MultihopMigrationState.OFF_TO_WHEN_NEEDED,
                        daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                        filtersSet = true,
                    ),
                userBlocked = false,
            )
        viewModel = init(multihopMigrationData = multihopMigrationData)

        // Act, Assert
        viewModel.uiState.test {
            // Initial state
            val initialState = awaitItem()
            assert(initialState.currentPageIndex == 0)
            // After nextPage
            viewModel.nextPage()
            val nextState = awaitItem()
            assert(nextState.currentPageIndex == 1)
            // After previousPage
            viewModel.previousPage()
            val previousState = awaitItem()
            assert(previousState.currentPageIndex == 0)
        }
    }

    @Test
    fun `when calling setEntryLocation should call repository setEntryLocation`() = runTest {
        // Arrange
        coEvery { mockWireguardConstraintsRepository.setEntryLocation(any()) } returns Unit.right()
        val multihopMigrationData =
            MultihopMigrationData(
                splitFilterMigration =
                    SplitFilterMigration(
                        multihopMigrationState = MultihopMigrationState.OFF_TO_WHEN_NEEDED,
                        daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                        filtersSet = true,
                    ),
                userBlocked = false,
            )
        viewModel = init(multihopMigrationData = multihopMigrationData)
        val entry = Constraint.Any

        // Act
        viewModel.setEntryLocation(entry)

        // Assert
        coVerify { mockWireguardConstraintsRepository.setEntryLocation(entry) }
    }

    @Test
    fun `when calling setMultihopMode should call repository setMultihopMode`() = runTest {
        // Arrange
        coEvery { mockWireguardConstraintsRepository.setMultihopMode(any()) } returns Unit.right()
        val multihopMigrationData =
            MultihopMigrationData(
                splitFilterMigration =
                    SplitFilterMigration(
                        multihopMigrationState = MultihopMigrationState.OFF_TO_WHEN_NEEDED,
                        daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                        filtersSet = true,
                    ),
                userBlocked = false,
            )
        viewModel = init(multihopMigrationData = multihopMigrationData)
        val mode = MultihopMode.WHEN_NEEDED

        // Act
        viewModel.setMultihopMode(mode)

        // Assert
        coVerify { mockWireguardConstraintsRepository.setMultihopMode(mode) }
    }

    @Test
    fun `when calling finishMigration should call repository clearMultihopMigrationState`() =
        runTest {
            // Arrange
            coEvery { mockMultihopMigrationRepository.clearMultihopMigrationState() } returns
                Unit.right()
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.OFF_TO_WHEN_NEEDED,
                            daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                            filtersSet = true,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act
            viewModel.finishMigration()

            // Assert
            coVerify { mockMultihopMigrationRepository.clearMultihopMigrationState() }
        }

    // This tests the pre-determined scenarios that we have for the multihop migration flow.
    // They are named similar with only the name of the scenario changing.
    @Test
    fun `when migration data is according to scenario 1b should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.OFF_TO_NEVER,
                            daitaMigration = PreviousDaitaState.OFF,
                            filtersSet = true,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assert(state.multihopMigrationPages.size == 3)
                assert(state.multihopMigrationPages[0] is MultihopMigrationPage.NewMultihopMode)
                assert(state.multihopMigrationPages[1] is MultihopMigrationPage.SeparateFilters)
                assert(state.multihopMigrationPages[2] is MultihopMigrationPage.SuggestedAction)
            }
        }

    @Test
    fun `when migration data is according to scenario 2 should have the correct pages`() = runTest {
        // Arrange
        val multihopMigrationData =
            MultihopMigrationData(
                splitFilterMigration =
                    SplitFilterMigration(
                        multihopMigrationState = MultihopMigrationState.OFF_TO_WHEN_NEEDED,
                        daitaMigration = PreviousDaitaState.ON,
                        filtersSet = false,
                    ),
                userBlocked = false,
            )
        viewModel = init(multihopMigrationData = multihopMigrationData)

        // Act, Assert
        viewModel.uiState.test {
            val state = awaitItem()
            assert(state.multihopMigrationPages.size == 1)
            assert(state.multihopMigrationPages[0] is MultihopMigrationPage.NewMultihopMode)
        }
    }

    @Test
    fun `when migration data is according to scenario 3a should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.OFF_TO_NEVER,
                            daitaMigration = PreviousDaitaState.ON,
                            filtersSet = true,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assert(state.multihopMigrationPages.size == 3)
                assert(state.multihopMigrationPages[0] is MultihopMigrationPage.NewMultihopMode)
                assert(state.multihopMigrationPages[1] is MultihopMigrationPage.SeparateFilters)
                assert(state.multihopMigrationPages[2] is MultihopMigrationPage.SuggestedAction)
            }
        }

    @Test
    fun `when migration data is according to scenario 3b should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.OFF_TO_ALWAYS,
                            daitaMigration = PreviousDaitaState.ON,
                            filtersSet = true,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assert(state.multihopMigrationPages.size == 3)
                assert(state.multihopMigrationPages[0] is MultihopMigrationPage.NewMultihopMode)
                assert(state.multihopMigrationPages[1] is MultihopMigrationPage.SeparateFilters)
                assert(
                    state.multihopMigrationPages[2] is MultihopMigrationPage.SuggestedMultihopEntry
                )
            }
        }

    @Test
    fun `when migration data is according to scenario 4a should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.OFF_TO_NEVER,
                            daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                            filtersSet = false,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assert(state.multihopMigrationPages.size == 2)
                assert(state.multihopMigrationPages[0] is MultihopMigrationPage.NewMultihopMode)
                assert(state.multihopMigrationPages[1] is MultihopMigrationPage.DirectOnlyRemoved)
            }
        }

    @Test
    fun `when migration data is according to scenario 4b should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.OFF_TO_NEVER,
                            daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                            filtersSet = false,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(2, state.multihopMigrationPages.size)
                assertIs<MultihopMigrationPage.NewMultihopMode>(state.multihopMigrationPages[0])
                assertIs<MultihopMigrationPage.DirectOnlyRemoved>(state.multihopMigrationPages[1])
            }
        }

    @Test
    fun `when migration data is according to scenario 5a should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.ON_TO_ALWAYS,
                            daitaMigration = PreviousDaitaState.OFF,
                            filtersSet = false,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(1, state.multihopMigrationPages.size)
                assertIs<MultihopMigrationPage.NewMultihopMode>(state.multihopMigrationPages[0])
            }
        }

    @Test
    fun `when migration data is according to scenario 5b should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.ON_TO_ALWAYS,
                            daitaMigration = PreviousDaitaState.OFF,
                            filtersSet = true,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(2, state.multihopMigrationPages.size)
                assertIs<MultihopMigrationPage.NewMultihopMode>(state.multihopMigrationPages[0])
                assertIs<MultihopMigrationPage.SeparateFilters>(state.multihopMigrationPages[1])
            }
        }

    @Test
    fun `when migration data is according to scenario 6a should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.ON_TO_ALWAYS,
                            daitaMigration = PreviousDaitaState.ON,
                            filtersSet = false,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(1, state.multihopMigrationPages.size)
                assertIs<MultihopMigrationPage.NewMultihopMode>(state.multihopMigrationPages[0])
            }
        }

    @Test
    fun `when migration data is according to scenario 6b should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.ON_TO_ALWAYS,
                            daitaMigration = PreviousDaitaState.ON,
                            filtersSet = true,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(3, state.multihopMigrationPages.size)
                assertIs<MultihopMigrationPage.NewMultihopMode>(state.multihopMigrationPages[0])
                assertIs<MultihopMigrationPage.SeparateFilters>(state.multihopMigrationPages[1])
                assertIs<MultihopMigrationPage.SuggestedMultihopEntry>(
                    state.multihopMigrationPages[2]
                )
            }
        }

    @Test
    fun `when migration data is according to scenario 7a should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.ON_TO_ALWAYS,
                            daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                            filtersSet = false,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(2, state.multihopMigrationPages.size)
                assertIs<MultihopMigrationPage.NewMultihopMode>(state.multihopMigrationPages[0])
                assertIs<MultihopMigrationPage.DirectOnlyRemoved>(state.multihopMigrationPages[1])
            }
        }

    @Test
    fun `when migration data is according to scenario 7b should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.ON_TO_ALWAYS,
                            daitaMigration = PreviousDaitaState.DIRECT_ONLY,
                            filtersSet = true,
                        ),
                    userBlocked = false,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assert(state.multihopMigrationPages.size == 4)
                assert(state.multihopMigrationPages[0] is MultihopMigrationPage.NewMultihopMode)
                assert(state.multihopMigrationPages[1] is MultihopMigrationPage.DirectOnlyRemoved)
                assert(state.multihopMigrationPages[2] is MultihopMigrationPage.SeparateFilters)
                assert(
                    state.multihopMigrationPages[3] is MultihopMigrationPage.SuggestedMultihopEntry
                )
            }
        }

    @Test
    fun `when migration data is according to catch-all-error scenario should have the correct pages`() =
        runTest {
            // Arrange
            val multihopMigrationData =
                MultihopMigrationData(
                    splitFilterMigration =
                        SplitFilterMigration(
                            multihopMigrationState = MultihopMigrationState.OFF_TO_NEVER,
                            daitaMigration = PreviousDaitaState.ON,
                            filtersSet = true,
                        ),
                    userBlocked = true,
                )
            viewModel = init(multihopMigrationData = multihopMigrationData)

            // Act, Assert
            viewModel.uiState.test {
                val state = awaitItem()
                assertEquals(3, state.multihopMigrationPages.size)
                assertIs<MultihopMigrationPage.NewMultihopMode>(state.multihopMigrationPages[0])
                assertIs<MultihopMigrationPage.SeparateFilters>(state.multihopMigrationPages[1])
                assertIs<MultihopMigrationPage.SuggestedAction>(state.multihopMigrationPages[2])
            }
        }

    private fun init(multihopMigrationData: MultihopMigrationData): MultihopMigrationViewModel =
        MultihopMigrationViewModel(
            navArgs = MultihopMigrationNavKey(multihopMigrationData),
            multihopMigrationRepository = mockMultihopMigrationRepository,
            wireguardConstraintsRepository = mockWireguardConstraintsRepository,
        )
}

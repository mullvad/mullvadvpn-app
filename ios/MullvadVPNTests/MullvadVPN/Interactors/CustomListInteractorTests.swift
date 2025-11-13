import Testing

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadSettings
@testable import MullvadTypes
@testable import WireGuardKitTypes

struct CustomListInteractorTests {
    static let store = InMemorySettingsStore<SettingNotFound>()

    init() {
        SettingsManager.unitTestStore = CustomListInteractorTests.store
    }

    private func makeDependencies() -> (
        customListInteractor: CustomListInteractor,
        tunnelManager: SettingsUpdatingMock
    ) {
        let tunnelManager = SettingsUpdatingMock()
        let customListInteractor = CustomListInteractor(
            tunnelManager: tunnelManager,
            repository: CustomListsRepositoryStub()
        )

        return (customListInteractor, tunnelManager)
    }

    @Test(
        "Adds custom list to repository"
    )
    func testAddCustomList() throws {
        let (customListInteractor, _) = makeDependencies()
        let customList = CustomList(name: "MyCustomList", locations: [])
        try? customListInteractor.save(list: customList)

        #expect(customListInteractor.fetch(by: customList.id) != nil)
    }

    @Test(
        "Add location to custom list"
    )
    func testAddLocationToCustomList() throws {
        let (customListInteractor, _) = makeDependencies()
        let customList = CustomList(name: "MyCustomList", locations: [])
        try? customListInteractor.save(list: customList)
        let location1 = RelayLocation.country("se")
        #expect(customListInteractor.fetch(by: customList.id)?.locations.isEmpty == true)

        try customListInteractor.addLocationToCustomList(relayLocations: [location1], customListName: customList.name)

        #expect(
            customListInteractor.fetch(by: customList.id)?.locations.first == location1
        )
    }

    @Test(
        "Custom list should not allow duplicate locations"
    )
    func testDoNotAddDuplicateLocations() throws {
        let (customListInteractor, _) = makeDependencies()
        let location1 = RelayLocation.country("se")
        let customList = CustomList(name: "MyCustomList", locations: [location1])
        try? customListInteractor.save(list: customList)

        #expect(
            customListInteractor.fetch(by: customList.id)?.locations.count == 1
        )

        try customListInteractor.addLocationToCustomList(relayLocations: [location1], customListName: customList.name)

        #expect(
            customListInteractor.fetch(by: customList.id)?.locations.count == 1
        )
    }

    @Test(
        "Removes a child location it the parent gets added to a custom list"
    )
    func testRemoveChildIfParentGetsAdded() throws {
        let (customListInteractor, _) = makeDependencies()
        let childLocation = RelayLocation.city("se", "got")
        let customList = CustomList(name: "MyCustomList", locations: [childLocation])
        try? customListInteractor.save(list: customList)

        let parentLocation = RelayLocation.country("se")

        try customListInteractor.addLocationToCustomList(
            relayLocations: [parentLocation],
            customListName: customList.name)

        #expect(
            customListInteractor.fetch(by: customList.id)?.locations.count == 1
        )
        #expect(
            customListInteractor.fetch(by: customList.id)?.locations.first == parentLocation
        )
    }

    @Test(
        "Remove location from custom list"
    )
    func testRemoveLocation() throws {
        let (customListInteractor, _) = makeDependencies()
        let location1 = RelayLocation.country("se")
        let customList = CustomList(name: "MyCustomList", locations: [location1])
        try? customListInteractor.save(list: customList)

        try customListInteractor.removeLocationFromCustomList(
            relayLocations: [location1],
            customListName: customList.name)

        #expect(
            customListInteractor.fetch(by: customList.id)?.locations.count == 0
        )
    }

    @Test(
        "If a list is selected as exit location and the list gets modified, the constraints should update"
    )
    func testUpdateConstraintsIfRemovedFromList() async throws {
        let (customListInteractor, tunnelManager) = makeDependencies()
        let location1 = RelayLocation.country("se")
        let customList = CustomList(name: "MyCustomList", locations: [location1])

        let selection = UserSelectedRelays(
            locations: [location1],
            customListSelection: .init(listId: customList.id, isList: true)
        )
        try? customListInteractor.save(list: customList)

        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.exitLocations = .only(
            selection
        )
        tunnelManager
            .updateSettings(
                [.relayConstraints(relayConstraints)],
                completionHandler: nil
            )
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations == .only(selection)
        )
        try? customListInteractor
            .removeLocationFromCustomList(
                relayLocations: [location1],
                customListName: customList.name
            )
        #expect(tunnelManager.updateCalled)
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations
                == .only(
                    .init(
                        locations: [],
                        customListSelection: selection.customListSelection
                    )
                )
        )
    }

    @Test(
        "The constraints should not update on custom list change if the list is not selected"
    )
    func testDoNotUpdateConstraintsIfSelectionNotAffected() async throws {
        let (customListInteractor, tunnelManager) = makeDependencies()
        let location1 = RelayLocation.country("se")
        let customList1 = CustomList(name: "MyCustomList1", locations: [location1])
        let customList2 = CustomList(name: "MyCustomList2", locations: [location1])

        try? customListInteractor.save(list: customList1)
        try? customListInteractor.save(list: customList2)

        let selection = UserSelectedRelays(
            locations: [location1],
            customListSelection: .init(listId: customList1.id, isList: true)
        )
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.exitLocations = .only(
            selection
        )
        tunnelManager.updateSettings(
            [.relayConstraints(relayConstraints)],
            completionHandler: nil
        )
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations == .only(selection)
        )
        tunnelManager.updateCalled = false

        try? customListInteractor
            .removeLocationFromCustomList(
                relayLocations: [location1],
                customListName: customList2.name
            )
        #expect(
            tunnelManager.updateCalled == false
        )
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations == .only(selection)
        )
    }

    @Test(
        "Removes the constraint when a custom list is removed"
    )
    func testRemoveConstraintIfListRemoved() async throws {
        let (customListInteractor, tunnelManager) = makeDependencies()
        let location1 = RelayLocation.country("se")
        let customList1 = CustomList(name: "MyCustomList1", locations: [location1])

        try? customListInteractor.save(list: customList1)

        let selection = UserSelectedRelays(
            locations: [location1],
            customListSelection: .init(listId: customList1.id, isList: true)
        )
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.exitLocations = .only(
            selection
        )
        tunnelManager.updateSettings(
            [.relayConstraints(relayConstraints)],
            completionHandler: nil
        )
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations == .only(selection)
        )
        tunnelManager.updateCalled = false

        customListInteractor
            .delete(customList: customList1)

        #expect(
            tunnelManager.updateCalled == true
        )
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations == .only(.init(locations: []))
        )
    }

    @Test(
        "Removes the constraint when a location inside a custom list is removed"
    )
    func testRemoveConstraintIfLocationRemoved() async throws {
        let (customListInteractor, tunnelManager) = makeDependencies()
        let location1 = RelayLocation.country("se")
        let location2 = RelayLocation.country("es")
        let customList1 = CustomList(name: "MyCustomList1", locations: [location1, location2])

        try? customListInteractor.save(list: customList1)

        let selection = UserSelectedRelays(
            locations: [location1],
            customListSelection: .init(listId: customList1.id, isList: false)
        )
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.exitLocations = .only(
            selection
        )
        tunnelManager.updateSettings(
            [.relayConstraints(relayConstraints)],
            completionHandler: nil
        )
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations == .only(selection)
        )
        tunnelManager.updateCalled = false

        try? customListInteractor
            .removeLocationFromCustomList(relayLocations: [location1], customListName: customList1.name)

        #expect(
            tunnelManager.updateCalled == true
        )
        #expect(
            tunnelManager.settings.relayConstraints.exitLocations == .only(.init(locations: []))
        )
    }
}

private class SettingsUpdatingMock: SettingsUpdating {
    var updateCalled = false
    func updateSettings(
        _ updates: [MullvadSettings.TunnelSettingsUpdate],
        completionHandler: (@Sendable () -> Void)?
    ) {
        for update in updates {
            update.apply(to: &settings)
        }
        updateCalled = true
    }

    var settings = LatestTunnelSettings()
}

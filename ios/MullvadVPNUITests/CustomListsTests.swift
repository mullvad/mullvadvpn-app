//
//  CustomListsTests.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2024-04-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class CustomListsTests: LoggedInWithTimeUITestCase {
    func testCreateCustomListPersistAfterAppRestarts() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        let customListName = createCustomListName()
        createCustomList(named: customListName)
        // Custom lists are persisted across app sessions, guarantee that the next test starts in a clean state
        addTeardownBlock {
            self.deleteCustomList(named: customListName)
        }

        app.terminate()
        app.launch()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()

        XCTAssertTrue(app.staticTexts[customListName].exists)
    }

    func testDeleteCustomList() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        let customListName = createCustomListName()
        createCustomList(named: customListName)
        deleteCustomList(named: customListName)

        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()

        XCTAssertFalse(app.staticTexts[customListName].exists)
    }

    func testEditCustomListLocations() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        let customListName = createCustomListName()
        createCustomList(named: customListName)

        addTeardownBlock {
            self.deleteCustomList(named: customListName)
        }

        startEditingCustomList(named: customListName)

        EditCustomListLocationsPage(app)
            .scrollToLocationWith(identifier: BaseUITestCase.testsDefaultCountryIdentifier)
            .toggleLocationCheckmarkWith(identifier: BaseUITestCase.testsDefaultCountryIdentifier)
            .tapBackButton()

        CustomListPage(app)
            .tapSaveListButton()

        ListCustomListsPage(app)
            .tapDoneButton()

        let customListItem = SelectLocationPage(app)
            .cellWithIdentifier(identifier: .locationListItem(customListName))

        XCTAssertTrue(customListItem.exists)
    }

    func testAddSingleLocationToCustomList() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        let customListName = createCustomListName()
        createCustomList(named: customListName)

        addTeardownBlock {
            self.deleteCustomList(named: customListName)
        }

        startEditingCustomList(named: customListName)

        EditCustomListLocationsPage(app)
            .scrollToLocationWith(identifier: BaseUITestCase.testsDefaultCountryIdentifier)
            .unfoldLocationwith(identifier: BaseUITestCase.testsDefaultCountryIdentifier)
            .unfoldLocationwith(identifier: BaseUITestCase.testsDefaultCityIdentifier)
            .toggleLocationCheckmarkWith(identifier: BaseUITestCase.testsDefaultRelayName)
            .tapBackButton()

        CustomListPage(app)
            .tapSaveListButton()

        ListCustomListsPage(app)
            .tapDoneButton()

        let customListLocation = SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: customListName)
            .cellWithIdentifier(identifier: .locationListItem(BaseUITestCase.testsDefaultRelayName))

        XCTAssertTrue(customListLocation.exists)
    }

    func createCustomList(named name: String) {
        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()
            .tapAddNewCustomList()

        // When creating a new custom list, the "create" button should be disabled until the list has a name at minimum
        CustomListPage(app)
            .verifyCreateButtonIs(enabled: false)
            .renameCustomList(name: name)
            .verifyCreateButtonIs(enabled: true)
            .tapCreateListButton()
    }

    func startEditingCustomList(named customListName: String) {
        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()
            .editExistingCustomLists()

        ListCustomListsPage(app)
            .selectCustomListToEdit(named: customListName)

        CustomListPage(app)
            .addOrEditLocations()

        EditCustomListLocationsPage(app)
    }

    func deleteCustomList(named customListName: String) {
        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()
            .editExistingCustomLists()

        ListCustomListsPage(app)
            .selectCustomListToEdit(named: customListName)

        CustomListPage(app)
            .deleteCustomList(named: customListName)
    }

    /// Creates a unique name for a custom list
    ///
    /// The name will be used as an accessibility identifier
    /// Those are lower case and case sensitive.
    func createCustomListName() -> String {
        let customListOriginalName = UUID().uuidString
        let index = customListOriginalName.index(customListOriginalName.startIndex, offsetBy: 30)
        return String(customListOriginalName.prefix(upTo: index)).lowercased()
    }
}

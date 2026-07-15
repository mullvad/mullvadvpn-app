//
//  CustomListsTests.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2024-04-17.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

class CustomListsTests: LoggedInWithTimeUITestCase {

    override class var settingsResetPolicy: UITestSettingsResetPolicy {
        .only([.customRelayLists, .recentConnections])
    }

    func testCreateCustomListPersistAfterAppRestarts() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        let customListName = createCustomListName()
        createCustomList(named: customListName)
        // Custom lists are persisted across app sessions, guarantee that the next test starts in a clean state
        addTeardownBlock {
            self.deleteCustomList(named: customListName)
        }

        try app.relaunch()

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
            .scrollToLocationWith(identifier: BaseUITestCase.testsDefaultCountryName)
            .toggleLocationCheckmarkWith(identifier: BaseUITestCase.testsDefaultCountryName)
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
            .scrollToLocationWith(identifier: BaseUITestCase.testsDefaultCountryName)
            .unfoldLocationwith(identifier: BaseUITestCase.testsDefaultCountryName)
            .unfoldLocationwith(identifier: BaseUITestCase.testsDefaultCityName)
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

    func testDeletingCustomListWithSingleEntryDoesNotCreateDuplicateInRecents() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        // MARK: 1) Create a custom list for setting up the test
        let customListName = createCustomListName()
        createCustomList(named: customListName)

        startEditingCustomList(named: customListName)

        EditCustomListLocationsPage(app)
            .scrollToLocationWith(identifier: BaseUITestCase.testsDefaultCountryName)
            .unfoldLocationwith(identifier: BaseUITestCase.testsDefaultCountryName)
            .unfoldLocationwith(identifier: BaseUITestCase.testsDefaultMullvadOwnedCityName)
            .toggleLocationCheckmarkWith(identifier: BaseUITestCase.testsDefaultMullvadOwnedRelayName)
            .tapBackButton()

        CustomListPage(app)
            .tapSaveListButton()

        ListCustomListsPage(app)
            .tapDoneButton()

        // MARK: 2) Connect to a relay that has the same item as in the custom list

        /// Guarantee that the "search" button doesn't eat input by scrolling to the bottom of the page
        /// Repeat the scroll every time a section is unfolded for the same reasons
        app.swipeUp(velocity: .fast)
        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultCountryName)
        app.swipeUp(velocity: .fast)
        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: BaseUITestCase.testsDefaultMullvadOwnedCityName)
        app.swipeUp(velocity: .fast)
        SelectLocationPage(app)
            .tapLocationCell(withName: BaseUITestCase.testsDefaultMullvadOwnedRelayName)

        allowAddVPNConfigurationsIfAsked()

        TunnelControlPage(app)
            .waitForConnectedLabel()
            .tapSelectLocationButton()

        // MARK: 3) Connect to the custom list
        SelectLocationPage(app)
            .tapLocationCell(withName: customListName)

        TunnelControlPage(app)
            .waitForConnectedLabel()
            .tapSelectLocationButton()

        // MARK: 4) Delete the custom list
        deleteCustomList(named: customListName)

        // MARK: 5) Verify that there is only 1 instance of the selected relay in the recents list
        ///
        /// `app.buttons[.recentListItem(BaseUITestCase.testsDefaultMullvadOwnedRelayName)]`
        /// cannot be used here as there are likely multiple invisible instances of such button.
        /// Hence the test resorts to count the number of list items for lack of better ways.
        XCTAssertTrue(
            app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "recentListItem")).count == 1)

        SelectLocationPage(app)
            .tapDoneButton()

        TunnelControlPage(app)
            .tapCancelOrDisconnectButton()
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

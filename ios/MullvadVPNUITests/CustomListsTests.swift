//
//  CustomListsTests.swift
//  MullvadVPNUITests
//
//  Created by Marco Nikic on 2024-04-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class CustomListsTests: LoggedInWithTimeUITestCase {
    func testCreateCustomListPersistAfterAppRestarts() throws {
        TunnelControlPage(app)
            .tapSelectLocationButton()

        let customListName = createCustomListName()
        createCustomList(named: customListName)

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

    func createCustomList(named name: String) {
        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()
            .tapCustomListEllipsisButton()
            .tapAddNewCustomList()

        // When creating a new custom list, the "create" button should be disabled until the list has a name at minimum
        CustomListPage(app)
            .verifyCreateButtonIs(enabled: false)
            .editCustomList(name: name)
            .verifyCreateButtonIs(enabled: true)
            .tapCreateListButton()
    }

    func deleteCustomList(named customListName: String) {
        SelectLocationPage(app)
            .tapWhereStatusBarShouldBeToScrollToTopMostPosition()
            .tapCustomListEllipsisButton()
            .editExistingCustomLists()

        ListCustomListsPage(app)
            .editCustomList(named: customListName)

        CustomListPage(app)
            .deleteCustomList(named: customListName)
    }

    func createCustomListName() -> String {
        let customListOriginalName = UUID().uuidString
        let index = customListOriginalName.index(customListOriginalName.startIndex, offsetBy: 30)
        return String(customListOriginalName.prefix(upTo: index))
    }
}

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

        SelectLocationPage(app)
            .openCustomListsActions()
            .tapAddNewCustomList()

        let customListName = createCustomListName()
        // When creating a new custom list, the "create" button should be disabled until the list has a name at minimum
        CustomListPage(app)
            .verifyCreateButtonIs(enabled: false)
            .editCustomList(name: customListName)
            .verifyCreateButtonIs(enabled: true)
            .tapCreateListButton()

        // Wait for the page to be shown again before quitting the app
        SelectLocationPage(app)

        app.terminate()
        app.launch()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .scrollToCustomListsSection()

        XCTAssertTrue(app.staticTexts[customListName].exists)
    }

    func createCustomListName() -> String {
        let customListOriginalName = UUID().uuidString
        let index = customListOriginalName.index(customListOriginalName.startIndex, offsetBy: 30)
        return String(customListOriginalName.prefix(upTo: index))
    }
}

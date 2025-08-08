import SwiftUI

extension Image {
    static var mullvadIconClose: some View { Image("IconClose")
        .resizable()
        .frame(width: 25, height: 25)
    }

    static let mullvadIconAlert = Image("IconAlert")
    static let mullvadIconSpinner = Image("IconSpinner")
    static let mullvadIconSuccess = Image("IconSuccess")
    static let mullvadIconFail = Image("IconFail")
    static let mullvadIconSearch = Image("IconSearch")
    static let mullvadIconCross = Image("IconCross")
}

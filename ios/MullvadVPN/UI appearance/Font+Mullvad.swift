import SwiftUI

extension Font {
    static let mullvadBig: Font = .largeTitle.bold()
    static let mullvadLarge: Font = .title.bold()
    static let mullvadMedium: Font = .title3.weight(.semibold)
    static let mullvadSmall: Font = .body
    static let mullvadSmallSemiBold: Font = mullvadSmall.weight(.semibold)
    static let mullvadTiny: Font = .subheadline
    static let mullvadTinySemiBold: Font = .mullvadTiny.weight(.semibold)
    static let mullvadMini: Font = .footnote
    static let mullvadMiniSemiBold: Font = mullvadMini.weight(.semibold)
}

extension UIFont {
    static let mullvadBig: UIFont = .preferredFont(forTextStyle: .largeTitle, weight: .bold)
    static let mullvadLarge: UIFont = .preferredFont(forTextStyle: .title1, weight: .bold)
    static let mullvadMedium: UIFont = .preferredFont(forTextStyle: .title3, weight: .semibold)
    static let mullvadSmall: UIFont = .preferredFont(forTextStyle: .body)
    static let mullvadSmallSemiBold: UIFont = .preferredFont(forTextStyle: .body, weight: .semibold)
    static let mullvadTiny: UIFont = .preferredFont(forTextStyle: .subheadline)
    static let mullvadTinySemiBold: UIFont = .preferredFont(forTextStyle: .subheadline, weight: .semibold)
    static let mullvadMini: UIFont = .preferredFont(forTextStyle: .footnote)
    static let mullvadMiniSemiBold: UIFont = .preferredFont(forTextStyle: .footnote, weight: .semibold)
}

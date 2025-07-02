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
    static var mullvadBig: UIFont { .preferredFont(forTextStyle: .largeTitle, weight: .bold) }
    static var mullvadLarge: UIFont { .preferredFont(forTextStyle: .title1, weight: .bold) }
    static var mullvadMedium: UIFont { .preferredFont(forTextStyle: .title3, weight: .semibold) }
    static var mullvadSmall: UIFont { .preferredFont(forTextStyle: .body) }
    static var mullvadSmallSemiBold: UIFont { .preferredFont(forTextStyle: .body, weight: .semibold) }
    static var mullvadTiny: UIFont { .preferredFont(forTextStyle: .subheadline) }
    static var mullvadTinySemiBold: UIFont { .preferredFont(forTextStyle: .subheadline, weight: .semibold) }
    static var mullvadMini: UIFont { .preferredFont(forTextStyle: .footnote) }
    static var mullvadMiniSemiBold: UIFont { .preferredFont(forTextStyle: .footnote, weight: .semibold) }
}

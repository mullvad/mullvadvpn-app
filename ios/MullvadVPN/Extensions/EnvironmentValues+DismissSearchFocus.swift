import SwiftUI

private struct DismissSearchFocusKey: EnvironmentKey {
    static let defaultValue: (@MainActor () -> Void)? = nil
}

extension EnvironmentValues {
    var dismissSearchFocus: (@MainActor () -> Void)? {
        get { self[DismissSearchFocusKey.self] }
        set { self[DismissSearchFocusKey.self] = newValue }
    }
}

struct LocationContext {
    var customListAvailableLocations: [LocationNode]
    var recents: [LocationNode]
    var locations: [LocationNode]
    var customLists: [LocationNode]
    var filter: [SelectLocationFilter]
    let selectLocation: (LocationNode) -> Void
    var totalRelayCount: Int
    var availableRelayCount: Int
    var selectedLocation: LocationNode?

    var isAutomaticLocation: Bool {
        selectedLocation is AutomaticLocationNode
    }

    init(
        recents: [LocationNode] = [],
        locations: [LocationNode] = [],
        customLists: [LocationNode] = [],
        filter: [SelectLocationFilter] = [],
        selectedLocation: LocationNode? = nil,
        connectedRelayHostname: String? = nil,
        totalRelayCount: Int = 0,
        availableRelayCount: Int = 0,
        selectLocation: @escaping (LocationNode) -> Void = { _ in }
    ) {
        self.recents = recents
        self.locations = locations
        self.customLists = customLists
        self.filter = filter
        self.totalRelayCount = totalRelayCount
        self.availableRelayCount = availableRelayCount
        self.selectLocation = selectLocation
        self.customListAvailableLocations = locations
    }

    var relaysAreFiltered: Bool {
        (availableRelayCount < totalRelayCount) && !isAutomaticLocation
    }
}

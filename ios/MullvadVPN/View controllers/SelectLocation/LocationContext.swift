struct LocationContext {
    var locations: [LocationNode]
    var customLists: [LocationNode]
    var filter: [SelectLocationFilter]
    let selectLocation: (LocationNode) -> Void
    var totalRelayCount: Int
    var availableRelayCount: Int

    init(
        locations: [LocationNode] = [],
        customLists: [LocationNode] = [],
        filter: [SelectLocationFilter] = [],
        selectedLocation: LocationNode? = nil,
        connectedRelayHostname: String? = nil,
        totalRelayCount: Int = 0,
        availableRelayCount: Int = 0,
        selectLocation: @escaping (LocationNode) -> Void = { _ in }
    ) {
        self.locations = locations
        self.customLists = customLists
        self.filter = filter
        self.totalRelayCount = totalRelayCount
        self.availableRelayCount = availableRelayCount
        self.selectLocation = selectLocation
    }

    var selectedLocation: LocationNode? {
        (locations + customLists)
            .flatMap { [$0] + $0.flattened }
            .first { $0.isSelected }
    }
}

extension LocationContext {
    var visibleLocations: [LocationNode] {
        locations.filter { !$0.isHiddenFromSearch }
    }
}

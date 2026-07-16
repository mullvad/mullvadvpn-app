import {
  Constraint,
  LiftedConstraint,
  Ownership,
  RelayLocation,
  RelayLocationGeographical,
  wrapConstraint,
} from '../../shared/daemon-rpc-types';
import {
  RelaySelectorPredicate,
  RelaySelectorPredicateEntryConstraints,
  RelaySelectorPredicateGeneralConstraints,
  RelaySelectorProvider,
} from '../../shared/relay-selector-rpc-types';
import {
  ISettingsReduxState,
  NormalRelaySettingsRedux,
  RelayLocationsFilterContext,
} from '../redux/settings/reducers';

function convertLocationToRelayLocationGeographical(
  location: LiftedConstraint<RelayLocation>,
): LiftedConstraint<RelayLocationGeographical> {
  if (location) {
    if (location !== 'any') {
      // NOTE: A CustomList is not a valid location for querying the Relay selector with.
      if (!('customList' in location)) {
        if ('hostname' in location) {
          return location;
        }

        if ('city' in location) {
          return location;
        }

        if ('country' in location) {
          return location;
        }
      }
    }
  }

  return 'any';
}

function convertProvidersToRelaySelectorProviders(providers: string[]): RelaySelectorProvider[] {
  return providers.map((provider) => ({ name: provider })) || [];
}

function convertLocationToRelaySelectorLocation(
  location: LiftedConstraint<RelayLocation>,
): Constraint<RelayLocationGeographical> {
  return wrapConstraint(convertLocationToRelayLocationGeographical(location));
}

function convertNormalRelaySettingsToRelaySelectorGeneralConstraints(
  normalRelaySettings: NormalRelaySettingsRedux,
  ignoreConstraint?: IgnoreConstraints,
): RelaySelectorPredicateGeneralConstraints {
  const location = convertLocationToRelaySelectorLocation(
    normalRelaySettings.wireguard.entryLocation,
  );
  const providers = convertProvidersToRelaySelectorProviders(normalRelaySettings.providers);
  const ownership = normalRelaySettings.ownership;

  return {
    location: ignoreConstraint?.location ? 'any' : location,
    providers: ignoreConstraint?.providers ? [] : providers,
    ownership: ignoreConstraint?.ownership ? Ownership.any : ownership,
  };
}

type IgnoreConstraints = {
  location?: boolean;
  ownership?: boolean;
  providers?: boolean;
};

type ConvertSettingsToRelaySelectorEntryConstraints = {
  settings: ISettingsReduxState;
  normalRelaySettings: NormalRelaySettingsRedux;
  ignoreConstraint?: IgnoreConstraints;
};

function convertSettingsToRelaySelectorEntryConstraints({
  settings,
  normalRelaySettings,
  ignoreConstraint: ignoreConstraint,
}: ConvertSettingsToRelaySelectorEntryConstraints): RelaySelectorPredicateEntryConstraints {
  const generalConstraints = convertNormalRelaySettingsToRelaySelectorGeneralConstraints(
    normalRelaySettings,
    ignoreConstraint,
  );

  const ipVersion = wrapConstraint(normalRelaySettings.wireguard.ipVersion);
  const daita = settings.wireguard.daita

  return {
    antiCensorship: settings.obfuscationSettings,
    daita,
    generalConstraints,
    ipVersion,
  };
}

type ConvertSettingsToRelaySelectorExitConstraints = {
  normalRelaySettings: NormalRelaySettingsRedux;
  ignoreConstraint?: IgnoreConstraints;
};

function convertSettingsToRelaySelectorExitConstraints({
  normalRelaySettings,
  ignoreConstraint: ignoreConstraint,
}: ConvertSettingsToRelaySelectorExitConstraints): RelaySelectorPredicateGeneralConstraints {
  const providers = convertProvidersToRelaySelectorProviders(normalRelaySettings.providers);
  const location = convertLocationToRelaySelectorLocation(normalRelaySettings.location);
  const ownership = normalRelaySettings.ownership;

  return {
    location: ignoreConstraint?.location ? 'any' : location,
    providers: ignoreConstraint?.providers ? [] : providers,
    ownership: ignoreConstraint?.ownership ? Ownership.any : ownership,
  };
}

type RelaySelectorQuery = {
  context: RelayLocationsFilterContext;
  predicate: RelaySelectorPredicate;
};

export function convertSettingsToRelaySelectorQueries(
  settings: ISettingsReduxState,
): RelaySelectorQuery[] | null {
  const normalRelaySettings =
    'normal' in settings.relaySettings ? settings.relaySettings.normal : null;

  if (!normalRelaySettings) {
    return null;
  }

  const multihop = normalRelaySettings?.wireguard.multihop;
  if (multihop === 'always') {
    // We perform 2 different queries here as there are performance
    // benefits to looking up both entry and exit queries in tandem.
    //
    // Explained in the context of looking at the Select location view:
    //
    // Say that you want to know how the currently selected Exit relay
    // affects the locations you can select from in the Entry view, then
    // you would need to also perform the same query from the Exit view's
    // perspective. When you switch to the other view you'd need both
    // perspectives again, but now inverse.
    //
    // As users often switch quickly between the Entry and Exit views
    // we don't want to cause any delay, hence we perform separate queries
    // from the Entry and Exit views' contexts.
    return [
      {
        // Use this when looking at the Entry view
        context: 'entry',
        predicate: {
          type: 'multihopEntry',
          constraints: {
            entry: convertSettingsToRelaySelectorEntryConstraints({
              normalRelaySettings,
              settings,
              ignoreConstraint: {
                location: true,
              },
            }),
            exit: convertSettingsToRelaySelectorExitConstraints({
              normalRelaySettings,
            }),
          },
        },
      },
      {
        // Use this when looking at the Exit view
        context: 'exit',
        predicate: {
          type: 'multihopExit',
          constraints: {
            entry: convertSettingsToRelaySelectorEntryConstraints({
              normalRelaySettings,
              settings,
            }),
            exit: convertSettingsToRelaySelectorExitConstraints({
              normalRelaySettings,
              ignoreConstraint: {
                location: true,
              },
            }),
          },
        },
      },
    ];
  }

  const entry = convertSettingsToRelaySelectorEntryConstraints({
    normalRelaySettings,
    settings,
    ignoreConstraint: {
      location: true,
    },
  });

  if (multihop === 'when-needed') {
    return [
      {
        // Use this when looking at the Select location view when multihop is set to when needed
        context: 'exit',
        predicate: {
          type: 'autohop',
          constraints: entry,
        },
      },
    ];
  }

  return [
    {
      // Use this when looking at the Select location view when multihop is set to never
      context: 'exit',
      predicate: {
        type: 'singlehop',
        constraints: entry,
      },
    },
  ];
}

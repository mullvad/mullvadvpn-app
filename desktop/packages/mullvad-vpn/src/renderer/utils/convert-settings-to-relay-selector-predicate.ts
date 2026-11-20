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

function convertSettingsToRelaySelectorGeneralConstraints({
  location,
  providers,
  ownership,
  ignoreConstraint,
}: {
  location: LiftedConstraint<RelayLocation>;
  providers: string[];
  ownership: Ownership;
  ignoreConstraint?: IgnoreConstraints;
}): RelaySelectorPredicateGeneralConstraints {
  return {
    location: ignoreConstraint?.location ? 'any' : convertLocationToRelaySelectorLocation(location),
    providers: ignoreConstraint?.providers
      ? []
      : convertProvidersToRelaySelectorProviders(providers),
    ownership: ignoreConstraint?.ownership ? Ownership.any : ownership,
  };
}

type IgnoreConstraints = {
  location?: boolean;
  ownership?: boolean;
  providers?: boolean;
};

type ConvertSettingsToRelaySelectorMultihopEntryConstraints = {
  settings: ISettingsReduxState;
  normalRelaySettings: NormalRelaySettingsRedux;
  ignoreConstraint?: IgnoreConstraints;
};

function convertSettingsToRelaySelectorMultihopEntryConstraints({
  settings,
  normalRelaySettings,
  ignoreConstraint,
}: ConvertSettingsToRelaySelectorMultihopEntryConstraints): RelaySelectorPredicateEntryConstraints {
  const { entryLocation, entryProviders, entryOwnership } = normalRelaySettings.wireguard;
  const generalConstraints = convertSettingsToRelaySelectorGeneralConstraints({
    location: entryLocation,
    providers: entryProviders,
    ownership: entryOwnership,
    ignoreConstraint,
  });

  const ipVersion = wrapConstraint(normalRelaySettings.wireguard.ipVersion);
  const daita = settings.wireguard.daita;

  return {
    antiCensorship: settings.obfuscationSettings,
    daita,
    generalConstraints,
    ipVersion,
  };
}

type ConvertSettingsToRelaySelectorMultihopExitConstraints = {
  normalRelaySettings: NormalRelaySettingsRedux;
  ignoreConstraint?: IgnoreConstraints;
};

function convertSettingsToRelaySelectorMultihopExitConstraints({
  normalRelaySettings,
  ignoreConstraint,
}: ConvertSettingsToRelaySelectorMultihopExitConstraints): RelaySelectorPredicateGeneralConstraints {
  const { location, providers, ownership } = normalRelaySettings;
  return convertSettingsToRelaySelectorGeneralConstraints({
    location,
    providers,
    ownership,
    ignoreConstraint,
  });
}

type ConvertSettingsToRelaySelectorOneHopConstraints = {
  settings: ISettingsReduxState;
  normalRelaySettings: NormalRelaySettingsRedux;
  ignoreConstraint?: IgnoreConstraints;
};

function convertSettingsToRelaySelectorOneHopConstraints({
  settings,
  normalRelaySettings,
  ignoreConstraint,
}: ConvertSettingsToRelaySelectorOneHopConstraints): RelaySelectorPredicateEntryConstraints {
  const { location, providers, ownership } = normalRelaySettings;
  const generalConstraints = convertSettingsToRelaySelectorGeneralConstraints({
    location,
    providers,
    ownership,
    ignoreConstraint,
  });

  const ipVersion = wrapConstraint(normalRelaySettings.wireguard.ipVersion);
  const daita = settings.wireguard.daita;

  return {
    antiCensorship: settings.obfuscationSettings,
    daita,
    generalConstraints,
    ipVersion,
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
            entry: convertSettingsToRelaySelectorMultihopEntryConstraints({
              normalRelaySettings,
              settings,
              ignoreConstraint: {
                location: true,
              },
            }),
            exit: convertSettingsToRelaySelectorMultihopExitConstraints({
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
            entry: convertSettingsToRelaySelectorMultihopEntryConstraints({
              normalRelaySettings,
              settings,
            }),
            exit: convertSettingsToRelaySelectorMultihopExitConstraints({
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

  if (multihop === 'when-needed') {
    return [
      {
        // Use this when looking at the Select location view when multihop is set to when needed
        context: 'exit',
        predicate: {
          type: 'autohop',
          constraints: convertSettingsToRelaySelectorOneHopConstraints({
            normalRelaySettings,
            settings,
            ignoreConstraint: {
              location: true,
            },
          }),
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
        constraints: convertSettingsToRelaySelectorOneHopConstraints({
          normalRelaySettings,
          settings,
          ignoreConstraint: {
            location: true,
          },
        }),
      },
    },
  ];
}

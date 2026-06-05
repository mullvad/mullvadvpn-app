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
  RelayPartitionsContext,
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
  unconstrain?: UnconstrainConstraints,
): RelaySelectorPredicateGeneralConstraints {
  const location = convertLocationToRelaySelectorLocation(
    normalRelaySettings.wireguard.entryLocation,
  );
  const providers = convertProvidersToRelaySelectorProviders(normalRelaySettings.providers);
  const ownership = normalRelaySettings.ownership;

  return {
    location: unconstrain?.location ? 'any' : location,
    providers: unconstrain?.providers ? [] : providers,
    ownership: unconstrain?.ownership ? Ownership.any : ownership,
  };
}

type UnconstrainConstraints = {
  location?: boolean;
  ownership?: boolean;
  providers?: boolean;
};

type ConvertSettingsToRelaySelectorEntryConstraints = {
  settings: ISettingsReduxState;
  normalRelaySettings: NormalRelaySettingsRedux;
  unconstrain?: UnconstrainConstraints;
};

function convertSettingsToRelaySelectorEntryConstraints({
  settings,
  normalRelaySettings,
  unconstrain,
}: ConvertSettingsToRelaySelectorEntryConstraints): RelaySelectorPredicateEntryConstraints {
  const generalConstraints = convertNormalRelaySettingsToRelaySelectorGeneralConstraints(
    normalRelaySettings,
    unconstrain,
  );

  const ipVersion = wrapConstraint(normalRelaySettings.wireguard.ipVersion);
  const daitaSettings = settings.wireguard.daita ?? { enabled: false, directOnly: false };

  return {
    antiCensorship: settings.obfuscationSettings,
    daitaSettings,
    generalConstraints,
    ipVersion,
  };
}

export type ConvertSettingsToRelaySelectorExitConstraints = {
  normalRelaySettings: NormalRelaySettingsRedux;
  unconstrain?: UnconstrainConstraints;
};

function convertSettingsToRelaySelectorExitConstraints({
  normalRelaySettings,
  unconstrain,
}: ConvertSettingsToRelaySelectorExitConstraints): RelaySelectorPredicateGeneralConstraints {
  const providers = convertProvidersToRelaySelectorProviders(normalRelaySettings.providers);
  const location = convertLocationToRelaySelectorLocation(normalRelaySettings.location);
  const ownership = normalRelaySettings.ownership;

  return {
    location: unconstrain?.location ? 'any' : location,
    providers: unconstrain?.providers ? [] : providers,
    ownership: unconstrain?.ownership ? Ownership.any : ownership,
  };
}

export type RelaySelectorQuery = {
  context: RelayPartitionsContext;
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

  const multihop = normalRelaySettings?.wireguard.useMultihop;
  if (multihop) {
    // We perform 3 different queries here as there are performance
    // benefits to looking up all in tandem. The 3 queries are:
    // entry, exit and filter.
    //
    // Explained in the context of looking at the Select location view:
    //
    // Say that you want to know how the currently selected Exit relay
    // affects the locations you can select from in the Entry view, then
    // you would need to also perform the same query from the Exit view's
    // perspective.
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
              unconstrain: {
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
              unconstrain: {
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
    unconstrain: {
      location: true,
    },
  });

  const daita = settings.wireguard.daita;
  const autohop = daita?.enabled && !daita?.directOnly;
  if (autohop) {
    return [
      {
        // Use this when looking at the Select location view with daita without direct only
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
      // Use this when looking at the Select location view without daita or with daita and direct only
      context: 'exit',
      predicate: {
        type: 'singlehop',
        constraints: entry,
      },
    },
  ];
}

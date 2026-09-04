import type { SplitFilterMigrationEvent } from '../../../../../shared/daemon-rpc-types';
import {
  DirectOnlyRemovedSlide,
  MultihopEntrySetToAutomaticSlide,
  NewMultihopModeSlide,
  SeparateFiltersSlide,
  SuggestedMultihopEntrySlide,
  SuggestedMultihopModeSlide,
} from '../components';

export function useMigrationSlides(migrationEvents: SplitFilterMigrationEvent[]) {
  if (migrationEvents.length === 0) {
    return null;
  }

  const scenario = migrationEvents[0].scenario;

  switch (scenario) {
    case 'one-b':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="off" to="never" />,
        <SeparateFiltersSlide key="separate-filters" />,
        <SuggestedMultihopModeSlide key="suggested-multihop-mode" />,
      ];

    case 'two':
      return [<NewMultihopModeSlide key="new-multihop-mode" from="off" to="when-needed" />];

    case 'three-a':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="off" to="never" />,
        <SeparateFiltersSlide key="separate-filters" />,
        <SuggestedMultihopModeSlide key="suggested-multihop-mode" />,
      ];

    case 'three-b':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="off" to="always" />,
        <SeparateFiltersSlide key="separate-filters" />,
        <SuggestedMultihopEntrySlide key="suggested-multihop-entry" />,
      ];

    case 'four-a':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="off" to="never" />,
        <DirectOnlyRemovedSlide key="direct-only-removed" />,
      ];

    case 'four-b':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="off" to="never" />,
        <DirectOnlyRemovedSlide key="direct-only-removed" />,
        <SeparateFiltersSlide key="separate-filters" />,
      ];

    case 'five-a':
      return [<NewMultihopModeSlide key="new-multihop-mode" from="on" to="always" />];

    case 'five-b':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="on" to="always" />,
        <SeparateFiltersSlide key="separate-filters" />,
      ];

    case 'six-a':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="on" to="always" />,
        <MultihopEntrySetToAutomaticSlide key="multihop-entry-set-to-automatic" />,
      ];

    case 'six-b':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="on" to="always" />,
        <SeparateFiltersSlide key="separate-filters" />,
        <SuggestedMultihopEntrySlide key="suggested-multihop-entry" />,
      ];

    case 'seven-a':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="on" to="always" />,
        <DirectOnlyRemovedSlide key="direct-only-removed" />,
      ];
    case 'seven-b':
      return [
        <NewMultihopModeSlide key="new-multihop-mode" from="on" to="always" />,
        <DirectOnlyRemovedSlide key="direct-only-removed" />,
        <SeparateFiltersSlide key="separate-filters" />,
        <SuggestedMultihopEntrySlide key="suggested-multihop-entry" />,
      ];

    default:
      return null;
  }
}

import { messages } from '../../../../../../../shared/gettext';
import type { FromMultihopMode, ToMultihopMode } from '../types';

export function getMultihopMigrationText(from: FromMultihopMode, to: ToMultihopMode) {
  if (from === 'off' && to === 'never') {
    return (
      // TRANSLATORS: Informs user that their multihop setting has been migrated from "Off" to "Never".
      messages.pgettext(
        'migrated-settings-view',
        'Your multihop setting was migrated from “Off” to “Never”.',
      )
    );
  } else if (from === 'off' && to === 'when-needed') {
    return (
      // TRANSLATORS: Informs user that their multihop setting has been migrated from "Off" to "When needed".
      messages.pgettext(
        'migrated-settings-view',
        'Your multihop setting was migrated from “Off” to “When needed”.',
      )
    );
  } else if (from === 'off' && to === 'always') {
    return (
      // TRANSLATORS: Informs user that their multihop setting has been migrated from "Off" to "Always".
      messages.pgettext(
        'migrated-settings-view',
        'Your multihop setting was migrated from “Off” to “Always”.',
      )
    );
  } else if (from === 'on' && to === 'always') {
    return (
      // TRANSLATORS: Informs user that their multihop setting has been migrated from "On" to "Always".
      messages.pgettext(
        'migrated-settings-view',
        'Your multihop setting was migrated from “On” to “Always”.',
      )
    );
  }
  return '';
}

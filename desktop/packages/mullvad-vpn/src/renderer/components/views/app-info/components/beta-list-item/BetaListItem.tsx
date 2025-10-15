import { messages } from '../../../../../../shared/gettext';
import { useSettingsShowBetaReleases, useVersionIsBeta } from '../../../../../redux/hooks';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function BetaListItem() {
  const { isBeta } = useVersionIsBeta();
  const { showBetaReleases, setShowBetaReleases } = useSettingsShowBetaReleases();

  return (
    <SettingsToggleListItem
      checked={showBetaReleases}
      onCheckedChange={setShowBetaReleases}
      disabled={isBeta}
      description={
        isBeta
          ? // TRANSLATORS: Description for beta program switch when using a beta version.
            messages.pgettext(
              'app-info-view',
              'This option is unavailable while using a beta version.',
            )
          : // TRANSLATORS: Description for beta program switch.
            messages.pgettext(
              'app-info-view',
              'Enable to get notified when new beta versions of the app are released.',
            )
      }>
      <SettingsToggleListItem.Label>
        {
          // TRANSLATORS: Label for switch to toggle beta program.
          messages.pgettext('app-info-view', 'Beta program')
        }
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export function ProblemReportButton() {
  // TRANSLATORS: Navigation button to the 'Report a problem' help view
  const label = messages.pgettext('support-view', 'Report a problem');

  return (
    <SettingsNavigationListItem to={RoutePath.problemReport}>
      <SettingsNavigationListItem.Label>{label}</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

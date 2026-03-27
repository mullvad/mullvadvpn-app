import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../shared/gettext';
import { Menu } from '../../../../../../lib/components/menu';
import type { MenuOptionProps } from '../../../../../../lib/components/menu-option';
import type { GeographicalLocation } from '../../../../types';

export type CreateCustomListMenuOptionProps = MenuOptionProps & {
  location: GeographicalLocation;
  onClick: () => void;
};

export function CreateCustomListMenuOption({
  onClick,
  location,
  ...props
}: CreateCustomListMenuOptionProps) {
  return (
    <Menu.Option {...props}>
      <Menu.Option.Trigger
        onClick={onClick}
        aria-label={sprintf(
          // TRANSLATORS: Label for button to create a new custom list with a specific location.
          // TRANSLATORS: Available placeholder:
          // TRANSLATORS: %(locationName)s - The name of the location being added to the list.
          messages.pgettext('custom-list-feature', 'Add %(locationName)s to new list'),
          {
            locationName: location.label,
          },
        )}>
        <Menu.Option.Item>
          <Menu.Option.Item.Icon icon="add" />
          <Menu.Option.Item.Label>{messages.gettext('New list')}</Menu.Option.Item.Label>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
  );
}

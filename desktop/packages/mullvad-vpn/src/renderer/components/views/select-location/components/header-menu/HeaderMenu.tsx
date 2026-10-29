import { messages } from '../../../../../../shared/gettext';
import { DisableRecentsDialog } from '../../../../../features/locations/components';
import { Menu, type MenuProps } from '../../../../../lib/components/menu';
import {
  MultihopAlwaysMenuOption,
  MultihopNeverMenuOption,
  MultihopWhenNeededMenuOption,
  RecentsMenuOption,
} from './components';
import { HeaderMenuProvider, useHeaderMenuContext } from './HeaderMenuContext';

export type HeaderMenuProps = MenuProps;

function HeaderMenuImpl({
  ...props
}: Omit<HeaderMenuProps, 'triggerRef' | 'open' | 'onOpenChange'>) {
  const { disableRecentsDialogOpen, open, onOpenChange, setDisableRecentsDialogOpen, triggerRef } =
    useHeaderMenuContext();

  return (
    <>
      <Menu open={open} onOpenChange={onOpenChange} triggerRef={triggerRef} {...props}>
        <Menu.Popup>
          <Menu.Title>
            {
              // TRANSLATORS: Title for a menu with items for the Multihop mode setting options.
              messages.pgettext('select-location-view', 'Multihop mode')
            }
          </Menu.Title>
          <MultihopAlwaysMenuOption />
          <MultihopWhenNeededMenuOption />
          <MultihopNeverMenuOption />
          <Menu.Divider />
          <RecentsMenuOption />
        </Menu.Popup>
      </Menu>
      <DisableRecentsDialog
        open={disableRecentsDialogOpen}
        onOpenChange={setDisableRecentsDialogOpen}
      />
    </>
  );
}

export function HeaderMenu({ onOpenChange, open, triggerRef, ...props }: HeaderMenuProps) {
  return (
    <HeaderMenuProvider onOpenChange={onOpenChange} open={open} triggerRef={triggerRef}>
      <HeaderMenuImpl {...props} />
    </HeaderMenuProvider>
  );
}

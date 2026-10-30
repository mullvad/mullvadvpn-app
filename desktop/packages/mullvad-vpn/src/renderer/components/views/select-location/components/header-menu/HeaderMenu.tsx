import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { DisableRecentsDialog } from '../../../../../features/locations/components';
import { EntryFiltersWarningDialog } from '../../../../../features/multihop/components';
import { Menu, type MenuProps } from '../../../../../lib/components/menu';
import {
  MultihopAlwaysMenuOption,
  MultihopNeverMenuOption,
  MultihopWhenNeededMenuOption,
  RecentsMenuOption,
} from './components';
import { HeaderMenuProvider, useHeaderMenuContext } from './HeaderMenuContext';
import { useSetMultihopWhenNeeded } from './hooks';

export type HeaderMenuProps = MenuProps;

function HeaderMenuImpl({
  ...props
}: Omit<HeaderMenuProps, 'triggerRef' | 'open' | 'onOpenChange'>) {
  const {
    disableRecentsDialogOpen,
    entryFiltersWarningDialogOpen,
    open,
    onOpenChange,
    setDisableRecentsDialogOpen,
    setEntryFiltersWarningDialogOpen,
    triggerRef,
  } = useHeaderMenuContext();
  const setMultihopWhenNeeded = useSetMultihopWhenNeeded();

  const handleContinue = React.useCallback(async () => {
    await setMultihopWhenNeeded();
    setEntryFiltersWarningDialogOpen(false);
    onOpenChange?.(false);
  }, [onOpenChange, setEntryFiltersWarningDialogOpen, setMultihopWhenNeeded]);

  const handleCancel = React.useCallback(() => {
    setEntryFiltersWarningDialogOpen(false);
  }, [setEntryFiltersWarningDialogOpen]);

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
      <EntryFiltersWarningDialog
        open={entryFiltersWarningDialogOpen}
        onCancel={handleCancel}
        onContinue={handleContinue}
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

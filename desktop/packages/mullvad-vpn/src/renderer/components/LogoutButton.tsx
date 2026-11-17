import { useCallback } from 'react';
import { Button } from '../lib/components';
import { useAppContext } from '../context';
import { messages } from '../../shared/gettext';

export function LogoutButton() {
  const { logout } = useAppContext();

  const onLogout = useCallback(async () => {
    await logout('gui-logout-button');
  }, [logout]);

  return (
    <Button color="destructive" onClick={onLogout}>
      <Button.Text>
        {
          // TRANSLATORS: Button label for logging out.
          messages.pgettext('account-view', 'Log out')
        }
      </Button.Text>
    </Button>
  );
}

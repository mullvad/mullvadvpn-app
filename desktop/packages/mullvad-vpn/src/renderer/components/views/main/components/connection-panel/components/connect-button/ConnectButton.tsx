import { useCallback } from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import log from '../../../../../../../../shared/logging';
import { useAppContext } from '../../../../../../../context';
import { Button, ButtonProps } from '../../../../../../../lib/components';

export function ConnectButton(props: ButtonProps) {
  const { connectTunnel } = useAppContext();

  const onConnect = useCallback(async () => {
    try {
      await connectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to connect the tunnel: ${error.message}`);
    }
  }, [connectTunnel]);

  return (
    <Button variant="success" onClick={onConnect} {...props}>
      <Button.Text>{messages.pgettext('tunnel-control', 'Connect')}</Button.Text>
    </Button>
  );
}

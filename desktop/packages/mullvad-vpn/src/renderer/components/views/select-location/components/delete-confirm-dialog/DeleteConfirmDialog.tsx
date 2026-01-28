import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { ICustomList } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { Button } from '../../../../../lib/components';
import { formatHtml } from '../../../../../lib/html-formatter';
import { ModalAlert, ModalAlertType, ModalMessage } from '../../../../Modal';

interface DeleteConfirmDialogProps {
  list: ICustomList;
  isOpen: boolean;
  hide: () => void;
  confirm: () => void;
}

// Dialog for changing the name of a custom list.
export function DeleteConfirmDialog(props: DeleteConfirmDialogProps) {
  const { confirm: propsConfirm, hide } = props;

  const confirm = useCallback(() => {
    propsConfirm();
    hide();
  }, [hide, propsConfirm]);

  return (
    <ModalAlert
      type={ModalAlertType.warning}
      isOpen={props.isOpen}
      buttons={[
        <Button key="save" variant="destructive" onClick={confirm}>
          <Button.Text>{messages.gettext('Delete list')}</Button.Text>
        </Button>,
        <Button key="cancel" onClick={props.hide}>
          <Button.Text>{messages.gettext('Cancel')}</Button.Text>
        </Button>,
      ]}
      close={props.hide}>
      <ModalMessage>
        {formatHtml(
          sprintf(
            messages.pgettext(
              'select-location-view',
              'Do you want to delete the list <b>%(list)s</b>?',
            ),
            { list: props.list.name },
          ),
        )}
      </ModalMessage>
    </ModalAlert>
  );
}

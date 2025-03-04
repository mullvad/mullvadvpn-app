import { messages } from '../../shared/gettext';
import { Button, IconButton, IconButtonProps } from '../lib/components';
import { useBoolean } from '../lib/utility-hooks';
import { ModalAlert, ModalAlertType } from './Modal';

export interface InfoButtonProps extends Omit<IconButtonProps, 'icon'> {
  title?: string;
  message?: string | Array<string>;
  children?: React.ReactNode;
}

export default function InfoButton({ title, message, children, ...props }: InfoButtonProps) {
  const [isOpen, show, hide] = useBoolean(false);

  return (
    <>
      <IconButton
        onClick={show}
        aria-label={messages.pgettext('accessibility', 'More information')}
        {...props}>
        <IconButton.Icon icon="info-circle" />
      </IconButton>
      <ModalAlert
        isOpen={isOpen}
        title={title}
        message={message}
        type={ModalAlertType.info}
        buttons={[
          <Button key="back" onClick={hide}>
            <Button.Text>{messages.gettext('Got it!')}</Button.Text>
          </Button>,
        ]}
        close={hide}>
        {children}
      </ModalAlert>
    </>
  );
}

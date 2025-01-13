import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { Button, IconButton, IconButtonProps } from '../lib/components';
import { useBoolean } from '../lib/utility-hooks';
import ImageView from './ImageView';
import { ModalAlert, ModalAlertType } from './Modal';

interface IInfoIconProps {
  className?: string;
  size?: number;
  tintColor?: string;
  tintHoverColor?: string;
}

export function InfoIcon(props: IInfoIconProps) {
  return (
    <ImageView
      source="icon-info"
      width={props.size ?? 18}
      tintColor={props.tintColor ?? colors.white}
      tintHoverColor={props.tintHoverColor ?? colors.white80}
      className={props.className}
    />
  );
}

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
        icon="icon-info"
        onClick={show}
        aria-label={messages.pgettext('accessibility', 'More information')}
        {...props}
      />
      <ModalAlert
        isOpen={isOpen}
        title={title}
        message={message}
        type={ModalAlertType.info}
        buttons={[
          <Button key="back" onClick={hide}>
            {messages.gettext('Got it!')}
          </Button>,
        ]}
        close={hide}>
        {children}
      </ModalAlert>
    </>
  );
}

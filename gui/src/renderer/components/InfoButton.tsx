import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useBoolean } from '../lib/utilityHooks';
import * as AppButton from './AppButton';
import ImageView from './ImageView';
import { ModalAlert, ModalAlertType } from './Modal';

const StyledInfoButton = styled.button({
  margin: '0 16px 0 0',
  borderWidth: 0,
  padding: 0,
  cursor: 'default',
  backgroundColor: 'transparent',
});

interface IInfoIconProps {
  className?: string;
  size?: number;
}

export function InfoIcon(props: IInfoIconProps) {
  return (
    <ImageView
      source="icon-info"
      width={props.size ?? 18}
      tintColor={colors.white}
      tintHoverColor={colors.white80}
      className={props.className}
    />
  );
}

interface IInfoButtonProps extends React.HTMLAttributes<HTMLButtonElement> {
  message?: string | Array<string>;
  children?: React.ReactNode;
  title?: string;
  size?: number;
}

export default function InfoButton(props: IInfoButtonProps) {
  const { message, children, size, ...otherProps } = props;
  const [isOpen, show, hide] = useBoolean(false);

  return (
    <>
      <StyledInfoButton
        onClick={show}
        aria-label={messages.pgettext('accessibility', 'More information')}
        {...otherProps}>
        <InfoIcon size={size} />
      </StyledInfoButton>
      <ModalAlert
        isOpen={isOpen}
        title={props.title}
        message={props.message}
        type={ModalAlertType.info}
        buttons={[
          <AppButton.BlueButton key="back" onClick={hide}>
            {messages.gettext('Got it!')}
          </AppButton.BlueButton>,
        ]}
        close={hide}>
        {props.children}
      </ModalAlert>
    </>
  );
}

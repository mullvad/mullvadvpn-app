import { messages } from '../../../../../../../../../../shared/gettext';
import { Button, type ButtonProps } from '../../../../../../../button';

export type WizardExitButtonProps = ButtonProps;

function WizardExitButton(props: WizardExitButtonProps) {
  return (
    <Button {...props}>
      <Button.Text>{messages.gettext('Got it!')}</Button.Text>
    </Button>
  );
}

const WizardExitButtonNamespace = Object.assign(WizardExitButton, {
  Text: Button.Text,
  Icon: Button.Icon,
});

export { WizardExitButtonNamespace as WizardExitButton };

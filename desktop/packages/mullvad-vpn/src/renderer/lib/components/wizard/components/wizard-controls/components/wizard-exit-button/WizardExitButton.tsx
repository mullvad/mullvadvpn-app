import { messages } from '../../../../../../../../shared/gettext';
import { Button, type ButtonProps } from '../../../../../button';
import { useSlides } from '../../../../../carousel/hooks';

export type WizardExitButtonProps = ButtonProps;

function WizardExitButton({ children, ...props }: WizardExitButtonProps) {
  const { isLastSlide } = useSlides();

  const show = isLastSlide;
  if (!show) {
    return null;
  }

  return (
    <Button {...props}>
      {children ? children : <Button.Text>{messages.gettext('Got it!')}</Button.Text>}
    </Button>
  );
}

const WizardExitButtonNamespace = Object.assign(WizardExitButton, {
  Text: Button.Text,
  Icon: Button.Icon,
});

export { WizardExitButtonNamespace as WizardExitButton };

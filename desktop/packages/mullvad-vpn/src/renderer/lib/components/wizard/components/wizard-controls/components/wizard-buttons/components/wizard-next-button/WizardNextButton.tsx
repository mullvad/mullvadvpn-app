import { messages } from '../../../../../../../../../../shared/gettext';
import { Button, type ButtonProps } from '../../../../../../../button';
import { useSlides } from '../../../../../../../carousel/hooks';

export type WizardNextButtonProps = ButtonProps;

function WizardNextButton(props: WizardNextButtonProps) {
  const { goToNextSlide } = useSlides();

  return (
    <Button onClick={goToNextSlide} {...props}>
      <Button.Text>{messages.gettext('Next')}</Button.Text>
      <Button.Icon icon="chevron-right" />
    </Button>
  );
}

const WizardNextButtonNamespace = Object.assign(WizardNextButton, {
  Text: Button.Text,
  Icon: Button.Icon,
});

export { WizardNextButtonNamespace as WizardNextButton };

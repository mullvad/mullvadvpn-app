import { messages } from '../../../../../../../../../../shared/gettext';
import { Button, type ButtonProps } from '../../../../../../../button';
import { useSlides } from '../../../../../../../carousel/hooks';

export type WizardPrevButtonProps = ButtonProps;

function WizardPrevButton(props: WizardPrevButtonProps) {
  const { goToPreviousSlide } = useSlides();

  return (
    <Button onClick={goToPreviousSlide} {...props}>
      <Button.Icon icon="chevron-left" />
      <Button.Text>{messages.gettext('Back')}</Button.Text>
    </Button>
  );
}

const WizardPrevButtonNamespace = Object.assign(WizardPrevButton, {
  Text: Button.Text,
  Icon: Button.Icon,
});

export { WizardPrevButtonNamespace as WizardPrevButton };

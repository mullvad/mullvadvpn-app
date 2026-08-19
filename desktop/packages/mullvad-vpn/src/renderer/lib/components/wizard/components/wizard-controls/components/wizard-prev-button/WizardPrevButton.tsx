import { messages } from '../../../../../../../../shared/gettext';
import { Button, type ButtonProps } from '../../../../../button';
import { useSlides } from '../../../../../carousel/hooks';

export type WizardPrevButtonProps = ButtonProps;

function WizardPrevButton({ children, ...props }: WizardPrevButtonProps) {
  const { goToPreviousSlide } = useSlides();

  const { isFirstSlide } = useSlides();

  const show = !isFirstSlide;
  if (!show) {
    return null;
  }

  return (
    <Button variant="secondary" onClick={goToPreviousSlide} {...props}>
      {children ? (
        children
      ) : (
        <>
          <Button.Icon icon="chevron-left" />
          <Button.Text>{messages.gettext('Back')}</Button.Text>
        </>
      )}
    </Button>
  );
}

const WizardPrevButtonNamespace = Object.assign(WizardPrevButton, {
  Text: Button.Text,
  Icon: Button.Icon,
});

export { WizardPrevButtonNamespace as WizardPrevButton };
